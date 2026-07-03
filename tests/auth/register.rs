#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn register_then_resolve_creates_user() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let v = value!({
        "data": {
            "email": "peter@example.com",
            "password": "Str0ngP@ssw0rd?",
        },
    });
    let r = exec_assert_ok(&s, Q_REGISTER, Some(v)).await;
    let r = r.data.to_json()?;

    let secret = r
        .pointer("/register/secret")
        .unwrap_or_default()
        .as_str()
        .unwrap_or_default();
    assert!(!secret.is_empty(), "secret should be in response");

    let t = AuthOtp::find().one_or_404(&d.tmp.db).await?;
    let v = value!({
        "data": {
            "id": t.id,
            "secret": secret,
            "otp": "999999",
        },
    });
    exec_assert_ok(&s, Q_REGISTER_RESOLVE, Some(v)).await;

    let u = User::find()
        .filter(UserColumn::Email.eq("peter@example.com"))
        .one_or_404(&d.tmp.db)
        .await?;
    assert!(
        rand_utils::password_eq(&u.password_hashed, "Str0ngP@ssw0rd?"),
        "password should be matched",
    );

    d.tmp.drop().await
}
