#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn register_resolve_creates_user_and_logs_them_in() -> Res<()> {
    let d = setup().await?;

    let (user_id, bearer) =
        register_and_resolve(&d, "olivia@fringe.example", "Zft-Cortexiphan-1985!", "042195").await?;

    let mut h = d.h.clone();
    h.insert(H_AUTHORIZATION, bearer);
    let s = d.schema(h);
    let q = "
    query {
        loginSessionCurrent {
            userId
        }
    }
    ";
    let expected = value!({
        "loginSessionCurrent": {
            "userId": user_id,
        },
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn register_reuses_pending_otp_within_cooldown() -> Res<()> {
    let d = setup().await?;
    let s = d.schema(d.h.clone());

    let q = r#"
    mutation {
        register(data: { email: "astrid@fringe.example", password: "Farnsworth-Lab-Assistant-77!" }) {
            id
        }
    }
    "#;
    exec_assert_ok(&s, q, None).await;

    let s = d.schema(d.h.clone());
    exec_assert_err(&s, q, None, &AuthErr::OtpReRequestTooSoon).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn register_with_already_registered_email_fails() -> Res<()> {
    let d = setup().await?;
    register_and_resolve(&d, "walter@fringe.example", "Cortexiphan-Trials-1991!", "119731").await?;

    let s = d.schema(d.h.clone());
    let q = r#"
    mutation {
        register(data: { email: "walter@fringe.example", password: "Another-Strong-Passphrase-42!" }) {
            id
        }
    }
    "#;
    exec_assert_err(&s, q, None, &SaasErr::RegisterEmailExists).await?;

    d.tmp.drop().await
}
