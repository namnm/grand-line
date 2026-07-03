#[path = "./setup.rs"]
mod setup;
use setup::*;

// Login with incorrect password returns LoginIncorrect error.
#[tokio::test]
async fn login_with_wrong_password_returns_login_incorrect() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let v = value!({
        "data": {
            "email": "olivia@example.com",
            "password": "wrongpassword",
        },
    });
    exec_assert_err(&s, Q_LOGIN, Some(v), &AuthErr::LoginIncorrect).await;

    d.tmp.drop().await
}
