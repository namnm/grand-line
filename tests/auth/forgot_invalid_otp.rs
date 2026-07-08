#[path = "./setup.rs"]
mod setup;
use setup::*;

// Submitting a wrong OTP during forgot-password resolve returns OtpResolveInvalid.
#[tokio::test]
async fn forgot_resolve_with_wrong_otp_returns_invalid() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    // Start a forgot-password flow to create an Otp row.
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

    // Resolve with the wrong OTP code.
    let v = value!({
        "data": {
            "id": t.id,
            "secret": secret,
            "otp": "000000",
        },
        "password": "NewStr0ng@Pass!",
    });
    exec_assert_err(&s, Q_FORGOT_RESOLVE, Some(v), &AuthErr::OtpResolveInvalid).await?;

    d.tmp.drop().await
}
