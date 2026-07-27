#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// #[auth] / #[auth(unauthenticated)] macro attribute
// ---------------------------------------------------------------------------

#[tokio::test]
async fn auth_macro_allows_authenticated_caller() -> Res<()> {
    let d = setup().await?;
    let user_id = ulid();
    let bearer = create_session(&d.tmp, &user_id).await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, bearer);
    let s = d.s.data(h).finish();

    let q = "
    query {
        ping
    }
    ";
    let expected = value!({
        "ping": true,
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn auth_macro_rejects_missing_token() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let q = "
    query {
        ping
    }
    ";
    exec_assert_err(&s, q, None, &AuthErr::Unauthenticated).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn auth_macro_rejects_wrong_secret() -> Res<()> {
    let d = setup().await?;
    let user_id = ulid();
    // Create a real session, but sign the header with a different secret.
    let _bearer = create_session(&d.tmp, &user_id).await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, h_bearer_for(&user_id, "wrong-secret"));
    let s = d.s.data(h).finish();

    let q = "
    query {
        ping
    }
    ";
    exec_assert_err(&s, q, None, &AuthErr::Unauthenticated).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn auth_macro_rejects_expired_session() -> Res<()> {
    let d = setup().await?;
    let user_id = ulid();
    let secret = rand_utils::secret();
    let sess = am_create!(Session {
        user_id: user_id.clone(),
        secret_hashed: rand_utils::secret_hash(&secret),
    })
    .exec_without_ctx(&d.tmp.db)
    .await?;

    // Push created_at well past the default cookie expiry window.
    let expires_ms = AuthConfig::default().cookie_login_session_expires_ms;
    Session::update_many()
        .filter_by_id(&sess.id)
        .col_expr(
            SessionColumn::CreatedAt,
            Expr::value(now() - duration_ms(expires_ms) - duration_ms(1000)),
        )
        .exec(&d.tmp.db)
        .await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, h_bearer_for(&sess.id, &secret));
    let s = d.s.data(h).finish();

    let q = "
    query {
        ping
    }
    ";
    exec_assert_err(&s, q, None, &AuthErr::Unauthenticated).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn auth_unauthenticated_macro_rejects_logged_in_caller() -> Res<()> {
    let d = setup().await?;
    let user_id = ulid();
    let bearer = create_session(&d.tmp, &user_id).await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, bearer);
    let s = d.s.data(h).finish();

    let q = "
    query {
        onlyWhenLoggedOut
    }
    ";
    exec_assert_err(&s, q, None, &AuthErr::AlreadyAuthenticated).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn auth_unauthenticated_macro_allows_logged_out_caller() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let q = "
    query {
        onlyWhenLoggedOut
    }
    ";
    let expected = value!({
        "onlyWhenLoggedOut": true,
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}
