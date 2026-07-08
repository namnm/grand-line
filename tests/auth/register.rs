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

    let secret = r.str("/register/secret");
    pretty_eq!(secret.is_empty(), false, "secret should be in response");

    let Some(t) = Otp::find().one(&d.tmp.db).await? else {
        return TestErr::expect("Otp row should be created by register");
    };

    let v = value!({
        "data": {
            "id": t.id,
            "secret": secret,
            "otp": "999999",
        },
    });
    exec_assert_ok(&s, Q_REGISTER_RESOLVE, Some(v)).await;

    let f = filter!(User {
        email: "peter@example.com",
    });
    let Some(u) = f.into_select().one(&d.tmp.db).await? else {
        return TestErr::expect("User row for peter@example.com should exist after register resolve");
    };

    pretty_eq!(
        rand_utils::password_eq(&u.password_hashed, "Str0ngP@ssw0rd?"),
        true,
        "password should be matched",
    );

    d.tmp.drop().await
}
