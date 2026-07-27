#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn forgot_resolve_changes_the_password() -> Res<()> {
    let d = setup().await?;
    register_and_resolve(&d, "john@fbi.example", "Original-Scott-Password-1!", "010203").await?;

    let s = d.schema(d.h.clone());
    let q = r#"
    mutation {
        forgot(data: { email: "john@fbi.example" }) {
            id
            secret
        }
    }
    "#;
    let res = exec_assert_ok(&s, q, None).await;
    let data = json_data(&res);
    let otp_id = data.str("/forgot/id").to_owned();
    let otp_secret = data.str("/forgot/secret").to_owned();

    known_otp_for(&d.tmp, &otp_id, "667788").await?;

    let s = d.schema(d.h.clone());
    let q = "
    mutation($id: String!, $secret: String!, $otp: String!, $password: String!) {
        forgotResolve(data: { id: $id, secret: $secret, otp: $otp }, password: $password) {
            id
        }
    }
    ";
    let v = value!({
        "id": otp_id,
        "secret": otp_secret,
        "otp": "667788",
        "password": "Brand-New-Scott-Password-2!",
    });
    exec_assert_ok(&s, q, Some(v)).await;

    let s = d.schema(d.h.clone());
    let q = r#"
    mutation {
        login(data: { email: "john@fbi.example", password: "Original-Scott-Password-1!" }) {
            id
        }
    }
    "#;
    exec_assert_err(&s, q, None, &SaasErr::LoginIncorrect).await?;

    let s = d.schema(d.h.clone());
    let q = r#"
    mutation {
        login(data: { email: "john@fbi.example", password: "Brand-New-Scott-Password-2!" }) {
            id
        }
    }
    "#;
    exec_assert_ok(&s, q, None).await;

    d.tmp.drop().await
}
