#[path = "./setup.rs"]
mod setup;
use setup::*;

#[tokio::test]
async fn login_with_correct_password_succeeds() -> Res<()> {
    let d = setup().await?;
    register_and_resolve(&d, "peter@fringe.example", "Amber-Universe-Bishop-11!", "205551").await?;

    let s = d.schema(d.h.clone());
    let q = r#"
    mutation {
        login(data: { email: "peter@fringe.example", password: "Amber-Universe-Bishop-11!" }) {
            id
            secret
        }
    }
    "#;
    exec_assert_ok(&s, q, None).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn login_with_wrong_password_fails() -> Res<()> {
    let d = setup().await?;
    register_and_resolve(
        &d,
        "nina@massivedynamic.example",
        "Parallel-Universe-Sharp-63!",
        "330218",
    )
    .await?;

    let s = d.schema(d.h.clone());
    let q = r#"
    mutation {
        login(data: { email: "nina@massivedynamic.example", password: "wrong-password-entirely" }) {
            id
        }
    }
    "#;
    exec_assert_err(&s, q, None, &SaasErr::LoginIncorrect).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn login_with_unknown_email_fails() -> Res<()> {
    let d = setup().await?;

    let s = d.schema(d.h.clone());
    let q = r#"
    mutation {
        login(data: { email: "broyles@fbi.example", password: "whatever-password-99!" }) {
            id
        }
    }
    "#;
    exec_assert_err(&s, q, None, &SaasErr::LoginIncorrect).await?;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// Real-world header gaps -- init_common_headers() always sets ideal ip/ua
// headers, these cases reproduce what an actual deployment can see: a client
// that sends no User-Agent at all, or a request carrying no x-socket-addr.
// ---------------------------------------------------------------------------

#[tokio::test]
async fn login_without_user_agent_header_succeeds() -> Res<()> {
    let d = setup().await?;
    register_and_resolve(&d, "broyles@fbi.example", "Division-Commander-Broyles-1!", "556677").await?;

    // a programmatic client sending no User-Agent is not a reason to refuse a login,
    // the session just records an empty one
    let mut h = d.h.clone();
    h.remove(H_UA);
    let s = d.schema(h);
    let q = r#"
    mutation {
        login(data: { email: "broyles@fbi.example", password: "Division-Commander-Broyles-1!" }) {
            inner {
                ua
            }
        }
    }
    "#;
    let expected = value!({
        "login": {
            "inner": {
                "ua": "{}",
            },
        },
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn login_without_any_ip_header_fails() -> Res<()> {
    let d = setup().await?;
    register_and_resolve(&d, "lincoln@fbi.example", "Fringe-Division-Lee-1!", "998877").await?;

    let mut h = d.h.clone();
    h.remove(H_SOCKET_ADDR);
    let s = d.schema(h);
    let q = r#"
    mutation {
        login(data: { email: "lincoln@fbi.example", password: "Fringe-Division-Lee-1!" }) {
            id
        }
    }
    "#;
    exec_assert_err(&s, q, None, &HttpErr::HeaderIp404).await?;

    d.tmp.drop().await
}

#[tokio::test]
async fn login_reads_the_socket_addr_not_a_client_supplied_header() -> Res<()> {
    let d = setup().await?;
    register_and_resolve(&d, "charlie@fbi.example", "Francis-Fringe-Division-1!", "443322").await?;

    // Mirrors main.rs's graphql_handler: it never sets x-real-ip/x-forwarded-for
    // itself, only x-socket-addr from axum's real ConnectInfo<SocketAddr>. A client
    // supplied x-real-ip must not win over it.
    let mut h = d.h.clone();
    h.insert(H_SOCKET_ADDR, h_str("203.0.113.42:54321"));
    h.insert(H_REAL_IP, h_str("10.0.0.1"));
    let s = d.schema(h);
    let q = r#"
    mutation {
        login(data: { email: "charlie@fbi.example", password: "Francis-Fringe-Division-1!" }) {
            inner {
                ip
            }
        }
    }
    "#;
    let expected = value!({
        "login": {
            "inner": {
                "ip": "203.0.113.42",
            },
        },
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}
