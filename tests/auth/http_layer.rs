use axum::Router;
use axum::body::Body;
use axum::extract::{ConnectInfo, Request};
use axum::http::{HeaderMap, HeaderValue};
use axum::middleware;
use axum::response::IntoResponse;
use axum::routing::post;
use core::net::SocketAddr;
use std::future::poll_fn;
use tower::Service;

#[path = "./setup.rs"]
mod setup;
use setup::*;

// ---------------------------------------------------------------------------
// The x-socket-addr layer: the framework-provided source for the default
// HttpIpSource::SocketAddr
// ---------------------------------------------------------------------------

async fn echo_socket_addr(headers: HeaderMap) -> impl IntoResponse {
    let v = headers
        .get(H_SOCKET_ADDR)
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned();
    [("x-echo", v)]
}

/// Drives the app once, the way into_make_service_with_connect_info does: the
/// connect info lands in the request extensions before the router sees it.
/// Returns the x-echo header the handler saw, i.e. the x-socket-addr value.
async fn run(router: Router, connect_info: Option<SocketAddr>, client_addr: Option<&str>) -> String {
    let req = Request::builder().method("POST").uri("/").body(Body::empty());
    let Some(mut req) = req.ok() else {
        return "<request failed to build>".to_owned();
    };
    if let Some(a) = connect_info {
        req.extensions_mut().insert(ConnectInfo(a));
    }
    if let Some(c) = client_addr {
        let Ok(v) = HeaderValue::from_str(c) else {
            return "<header value failed to build>".to_owned();
        };
        req.headers_mut().insert(H_SOCKET_ADDR, v);
    }

    // Router::poll_ready is always ready, call it once to honor the service
    // contract, then issue the request. Router's error type is Infallible.
    let mut svc = router;
    let _ = poll_fn(|cx| <Router as Service<Request>>::poll_ready(&mut svc, cx)).await;
    let res = match <Router as Service<Request>>::call(&mut svc, req).await {
        Ok(res) => res,
        Err(e) => match e {},
    };
    res.headers()
        .get("x-echo")
        .and_then(|v| v.to_str().ok())
        .unwrap_or_default()
        .to_owned()
}

fn app_with_layer() -> Router {
    Router::new()
        .route("/", post(echo_socket_addr))
        .layer(middleware::from_fn(socket_addr_layer))
}

fn local_addr() -> SocketAddr {
    SocketAddr::from(([127, 0, 0, 1], 8080))
}

#[tokio::test]
async fn the_layer_sets_x_socket_addr_from_connect_info() {
    let got = run(app_with_layer(), Some(local_addr()), None).await;
    pretty_eq!(
        got,
        "127.0.0.1:8080",
        "the layer should fill x-socket-addr from the real connection"
    );
}

#[tokio::test]
async fn the_layer_overrides_a_client_supplied_x_socket_addr() {
    let got = run(app_with_layer(), Some(local_addr()), Some("6.6.6.6:9")).await;
    pretty_eq!(
        got,
        "127.0.0.1:8080",
        "a client supplied x-socket-addr must not win over the socket"
    );
}

#[tokio::test]
async fn requests_without_connect_info_pass_through() {
    let got = run(app_with_layer(), None, None).await;
    pretty_eq!(
        got,
        "",
        "no connect info, nothing to derive the header from, request passes through"
    );
}

// Without the layer the header is never filled from the socket: the same
// router without it only sees what the client sent, which is the gap the
// layer exists to close.
#[tokio::test]
async fn without_the_layer_nothing_fills_the_header() {
    let app = Router::new().route("/", post(echo_socket_addr));
    let got = run(app, Some(local_addr()), None).await;
    pretty_eq!(got, "", "without the layer the connect info never reaches the header");
}
