use crate::prelude::*;

/// Read-only session lookup backing framework auth engine.
pub struct SaasAuthSessionImpl;
#[async_trait]
impl AuthSessionImpl for SaasAuthSessionImpl {
    async fn find(&self, ctx: &Context<'_>, id: &str) -> Res<Option<AuthImplSession>> {
        let tx = &*ctx.tx().await?;

        let Some(ls) = LoginSession::find()
            .include_deleted(false)
            .filter_by_id(id)
            .one(tx)
            .await?
        else {
            return Ok(None);
        };

        Ok(Some(AuthImplSession {
            id: ls.id,
            user_id: ls.user_id,
            secret_hashed: ls.secret_hashed,
            created_at: ls.created_at,
        }))
    }
}

/// Otp lookup and mutations backing framework auth engine.
/// Using db to persist immediately even when the overall request later errors,
/// and not mixing with tx to avoid deadlock.
pub struct SaasAuthOtpImpl;
#[async_trait]
impl AuthOtpImpl for SaasAuthOtpImpl {
    async fn find(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<Option<AuthImplOtp>> {
        let db = ctx.db().await?;
        let t = Otp::find()
            .include_deleted(false)
            .filter(OtpColumn::Ty.eq(ty))
            .filter(OtpColumn::Email.eq(email))
            .one(db)
            .await?;
        Ok(t.map(Into::into))
    }

    async fn increment(&self, ctx: &Context<'_>, id: &str, ty: &str) -> Res<Option<AuthImplOtp>> {
        let db = ctx.db().await?;
        let u = Otp::update_many()
            .include_deleted(false)
            .filter_by_id(id)
            .filter(OtpColumn::Ty.eq(ty))
            .set(OtpActiveModel::defaults_on_update())
            .col_expr(OtpColumn::TotalAttempt, Expr::col(OtpColumn::TotalAttempt).add(1));

        if u.exec(db).await?.rows_affected == 0 {
            return Ok(None);
        }
        let t = Otp::find().include_deleted(false).filter_by_id(id).one(db).await?;
        Ok(t.map(Into::into))
    }

    async fn reset(&self, ctx: &Context<'_>, id: &str) -> Res<()> {
        let db = ctx.db().await?;
        Otp::update_many()
            .filter_by_id(id)
            .col_expr(OtpColumn::TotalAttempt, Expr::value(0))
            .exec(db)
            .await?;
        Ok(())
    }

    async fn delete(&self, ctx: &Context<'_>, ty: &str, email: &str) -> Res<()> {
        let db = ctx.db().await?;
        Otp::delete_many()
            .filter(OtpColumn::Ty.eq(ty))
            .filter(OtpColumn::Email.eq(email))
            .exec(db)
            .await?;
        Ok(())
    }
}

impl From<OtpSql> for AuthImplOtp {
    fn from(t: OtpSql) -> Self {
        Self {
            id: t.id,
            email: t.email,
            secret_hashed: t.secret_hashed,
            otp_salt: t.otp_salt,
            otp_hashed: t.otp_hashed,
            total_attempt: t.total_attempt,
            created_at: t.created_at,
            data: t.data,
        }
    }
}
