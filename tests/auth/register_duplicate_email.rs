#[path = "./setup.rs"]
mod setup;
use setup::*;

// Registering with an email that is already in use returns RegisterEmailExists.
#[tokio::test]
async fn register_with_existing_email_returns_email_exists() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    // olivia@example.com is already seeded in setup().
    let v = value!({
        "data": {
            "email": "olivia@example.com",
            "password": "Str0ngP@ssw0rd?",
        },
    });
    exec_assert_err(&s, Q_REGISTER, Some(v), &AuthErr::RegisterEmailExists).await?;

    d.tmp.drop().await
}
