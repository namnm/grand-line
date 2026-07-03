#[path = "./setup.rs"]
mod setup;
use setup::*;

// Login with an email that does not exist returns LoginIncorrect error
// (same error as wrong password to avoid user enumeration).
#[tokio::test]
async fn login_with_unknown_email_returns_login_incorrect() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let v = value!({
        "data": {
            "email": "nobody@example.com",
            "password": "123123",
        },
    });
    exec_assert_err(&s, Q_LOGIN, Some(v), &AuthErr::LoginIncorrect).await;

    d.tmp.drop().await
}
