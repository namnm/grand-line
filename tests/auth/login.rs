#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn login_with_correct_credentials_returns_session() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let v = value!({
        "data": {
            "email": "olivia@example.com",
            "password": "123123",
        },
    });
    let expected = value!({
        "login": {
            "inner": {
                "userId": d.user_id,
            },
        },
    });
    exec_assert(&s, Q_LOGIN, Some(v), &expected).await;

    d.tmp.drop().await
}
