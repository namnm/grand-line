#[path = "./setup.rs"]
mod setup;
use setup::*;

// Calling login while already authenticated returns AlreadyAuthenticated.
#[tokio::test]
async fn login_while_authenticated_returns_already_authenticated() -> Res<()> {
    let d = setup().await?;

    let mut h = d.h;
    h.insert(H_AUTHORIZATION, h_bearer(&d.token));
    let s = d.s.data(h).finish();

    let v = value!({
        "data": {
            "email": "olivia@example.com",
            "password": "123123",
        },
    });
    exec_assert_err(&s, Q_LOGIN, Some(v), &AuthErr::AlreadyAuthenticated).await?;

    d.tmp.drop().await
}
