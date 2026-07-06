#[path = "./setup.rs"]
mod setup;
use setup::*;

// Submitting a wrong OTP during registration resolve returns OtpResolveInvalid.
#[tokio::test]
async fn register_resolve_with_wrong_otp_returns_invalid() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    // First, start a registration to create an AuthOtp row.
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

    let Some(t) = AuthOtp::find().one(&d.tmp.db).await? else {
        return TestErr::expect("AuthOtp row should be created by register");
    };

    // Resolve with the wrong OTP code.
    let v = value!({
        "data": {
            "id": t.id,
            "secret": secret,
            "otp": "000000",
        },
    });
    exec_assert_err(&s, Q_REGISTER_RESOLVE, Some(v), &AuthErr::OtpResolveInvalid).await?;

    d.tmp.drop().await
}
