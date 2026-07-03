use crate::prelude::*;

#[gql_input]
pub struct Login {
    pub email: String,
    pub password: String,
}

pub struct LoginSessionData {
    pub ip: String,
    pub ua: HashMap<String, String>,
}

pub async fn login_impl<U>(ctx: &Context<'_>, data: Login) -> Res<LoginSessionWithSecret>
where
    U: AuthUser,
{
    ctx.auth_ensure_not_authenticated().await?;

    let tx = &*ctx.tx().await?;
    let lsd = ctx.login_session_data()?;
    let h = &ctx.auth_config().handlers;

    let u = U::find()
        .include_deleted(false)
        .filter(U::email_col().eq(&data.email))
        .one(tx)
        .await?
        .ok_or(MyErr::LoginIncorrect)?;

    if !rand_utils::password_eq(U::get_password_hashed(&u), &data.password) {
        return Err(MyErr::LoginIncorrect.into());
    }

    let ls = login_session_create(ctx, tx, &u.get_id(), &lsd).await?;

    h.on_login_resolve(ctx, &u.get_id(), &ls.inner).await?;

    Ok(ls)
}

pub async fn login_session_create(
    ctx: &Context<'_>,
    tx: &DatabaseTransaction,
    user_id: &str,
    data: &LoginSessionData,
) -> Res<LoginSessionWithSecret> {
    let secret = rand_utils::secret();
    let ls = am_create!(LoginSession {
        user_id: user_id.to_owned(),
        secret_hashed: rand_utils::secret_hash(&secret),
        ip: data.ip.clone(),
        ua: data.ua.to_json()?,
    })
    .exec_without_ctx(tx)
    .await?;

    let lsws = LoginSessionWithSecret {
        inner: ls,
        secret: secret.clone(),
    };
    ctx.set_cookie_login_session(&lsws)?;
    Ok(lsws)
}
