#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn forgot_then_resolve_updates_password() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let v = value!({
        "data": {
            "email": "olivia@example.com",
        },
    });
    let r = exec_assert_ok(&s, Q_FORGOT, Some(v)).await;
    let r = r.data.to_json()?;

    let secret = r.str("/forgot/secret");
    pretty_eq!(secret.is_empty(), false, "secret should be in response");

    let Some(t) = Otp::find().one(&d.tmp.db).await? else {
        return TestErr::expect("Otp row should be created by forgot");
    };

    let v = value!({
        "data": {
            "id": t.id,
            "secret": secret,
            "otp": "999999",
        },
        "password": "Str0ngP@ssw0rd?",
    });
    let expected = value!({
        "forgotResolve": {
            "inner": {
                "userId": d.user_id,
            },
        },
    });
    exec_assert(&s, Q_FORGOT_RESOLVE, Some(v), &expected).await;

    let f = filter!(User {
        email: "olivia@example.com",
    });
    let Some(u) = f.into_select().one(&d.tmp.db).await? else {
        return TestErr::expect("User row for olivia@example.com should exist after forgot resolve");
    };

    let password_eq = rand_utils::password_eq(&u.password_hashed, "Str0ngP@ssw0rd?");
    pretty_eq!(password_eq, true, "password should be updated");

    d.tmp.drop().await
}
