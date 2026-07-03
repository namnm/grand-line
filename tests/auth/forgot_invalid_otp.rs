#[path = "./setup.rs"]
mod setup;
use setup::*;

// Submitting a wrong OTP during forgot-password resolve returns OtpResolveInvalid.
#[tokio::test]
async fn forgot_resolve_with_wrong_otp_returns_invalid() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    // Start a forgot-password flow to create an AuthOtp row.
    let v = value!({
        "data": {
            "email": "olivia@example.com",
        },
    });
    let r = exec_assert_ok(&s, Q_FORGOT, Some(v)).await;
    let r = r.data.to_json()?;

    let secret = r
        .pointer("/forgot/secret")
        .unwrap_or_default()
        .as_str()
        .unwrap_or_default();
    assert!(!secret.is_empty(), "secret should be in response");

    let t = AuthOtp::find().one_or_404(&d.tmp.db).await?;

    // Resolve with the wrong OTP code.
    let v = value!({
        "data": {
            "id": t.id,
            "secret": secret,
            "otp": "000000",
        },
        "password": "NewStr0ng@Pass!",
    });
    exec_assert_err(&s, Q_FORGOT_RESOLVE, Some(v), &AuthErr::OtpResolveInvalid).await;

    d.tmp.drop().await
}
