#[path = "./setup.rs"]
mod setup;
use axum::http::HeaderValue;
use setup::*;

fn set_cookie_header(r: &Response) -> String {
    r.http_headers
        .get(H_SET_COOKIE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

// ---------------------------------------------------------------------------
// get_ip reads only the configured source
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ip_defaults_to_the_socket_addr() -> Res<()> {
    let d = setup().await?;

    let mut h = d.h;
    h.insert(H_SOCKET_ADDR, h_str("198.51.100.7:41234"));
    let s = d.s.data(h).finish();

    let q = "
    query {
        currentIp
    }
    ";
    let expected = value!({
        "currentIp": "198.51.100.7",
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn ip_ignores_a_client_supplied_header_by_default() -> Res<()> {
    let d = setup().await?;

    // a client reachable directly can send any x-real-ip it likes, the default
    // source must not read it
    let mut h = d.h;
    h.insert(H_SOCKET_ADDR, h_str("198.51.100.7:41234"));
    h.insert(H_REAL_IP, h_str("10.0.0.1"));
    let s = d.s.data(h).finish();

    let q = "
    query {
        currentIp
    }
    ";
    let expected = value!({
        "currentIp": "198.51.100.7",
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn ip_reads_the_configured_proxy_header() -> Res<()> {
    let d = setup().await?;

    let c = HttpConfig {
        ip_source: HttpIpSource::Proxy {
            header: H_REAL_IP,
            hops: 1,
        },
        ..Default::default()
    };
    let mut h = d.h;
    h.insert(H_REAL_IP, h_str("203.0.113.9"));
    let s = d.s.data(h).data(c).finish();

    let q = "
    query {
        currentIp
    }
    ";
    let expected = value!({
        "currentIp": "203.0.113.9",
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn ip_counts_proxy_hops_from_the_right() -> Res<()> {
    let d = setup().await?;

    // x-forwarded-for grows to the right, with two trusted proxies the client is
    // the second entry from the right, and the leading entry is client supplied
    let c = HttpConfig {
        ip_source: HttpIpSource::Proxy {
            header: H_FORWARDED_FOR,
            hops: 2,
        },
        ..Default::default()
    };
    let mut h = d.h;
    h.insert(H_FORWARDED_FOR, h_str("1.2.3.4, 203.0.113.9, 192.0.2.5"));
    let s = d.s.data(h).data(c).finish();

    let q = "
    query {
        currentIp
    }
    ";
    let expected = value!({
        "currentIp": "203.0.113.9",
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn ip_errors_when_the_configured_source_is_absent() -> Res<()> {
    let d = setup().await?;

    let mut h = d.h;
    h.remove(H_SOCKET_ADDR);
    let s = d.s.data(h).finish();

    let q = "
    query {
        currentIp
    }
    ";
    exec_assert_err(&s, q, None, &HttpErr::HeaderIp404).await?;

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// get_ua no longer refuses a request over a missing User-Agent
// ---------------------------------------------------------------------------

#[tokio::test]
async fn ua_is_empty_when_the_header_is_absent() -> Res<()> {
    let d = setup().await?;

    let mut h = d.h;
    h.remove(H_UA);
    h.remove(H_UA_SEC_CH);
    let s = d.s.data(h).finish();

    let q = "
    query {
        currentUa
    }
    ";
    let expected = value!({
        "currentUa": "{}",
    });
    exec_assert(&s, q, None, &expected).await;

    d.tmp.drop().await
}

#[tokio::test]
async fn ua_still_reports_the_header_when_present() -> Res<()> {
    let d = setup().await?;

    let mut h = d.h;
    h.insert(H_UA, HeaderValue::from_static("Observer/1.0"));
    h.remove(H_UA_SEC_CH);
    let s = d.s.data(h).finish();

    let q = "
    query {
        currentUa
    }
    ";
    let r = exec_assert_ok(&s, q, None).await;
    let r = r.data.to_json()?;

    pretty_eq!(
        r.str("/currentUa").contains("Observer/1.0"),
        true,
        "a present user agent should still be reported",
    );

    d.tmp.drop().await
}

// ---------------------------------------------------------------------------
// set_cookie writes SameSite and Path
// ---------------------------------------------------------------------------

#[tokio::test]
async fn cookie_defaults_to_same_site_lax_and_root_path() -> Res<()> {
    let d = setup().await?;
    let s = d.s.data(d.h).finish();

    let q = "
    mutation {
        cookieTest
    }
    ";
    let r = exec_assert_ok(&s, q, None).await;
    let c = set_cookie_header(&r);

    pretty_eq!(
        c.contains("SameSite=Lax"),
        true,
        "default cookie should be SameSite=Lax, got {c}"
    );
    pretty_eq!(
        c.contains("Path=/"),
        true,
        "default cookie should be scoped to Path=/, got {c}"
    );

    d.tmp.drop().await
}

#[tokio::test]
async fn cookie_follows_the_configured_same_site_and_path() -> Res<()> {
    let d = setup().await?;

    // what a browser app on a different origin than the api needs
    let c = HttpConfig {
        cookie_same_site: SameSite::None,
        cookie_path: "/api",
        ..Default::default()
    };
    let s = d.s.data(d.h).data(c).finish();

    let q = "
    mutation {
        cookieTest
    }
    ";
    let r = exec_assert_ok(&s, q, None).await;
    let c = set_cookie_header(&r);

    pretty_eq!(
        c.contains("SameSite=None"),
        true,
        "cookie should follow the configured SameSite, got {c}"
    );
    pretty_eq!(
        c.contains("Path=/api"),
        true,
        "cookie should follow the configured Path, got {c}"
    );
    pretty_eq!(
        c.contains("Secure"),
        true,
        "SameSite=None is only accepted together with Secure, got {c}"
    );

    d.tmp.drop().await
}
