use crate::prelude::*;
use axum::extract::{ConnectInfo, Request};
use axum::http::HeaderValue;
use axum::middleware::Next;
use axum::response::Response;
use core::net::SocketAddr;

/// The header get_ip reads by default, see HttpIpSource::SocketAddr. Defined
/// here because _http_axum does not depend on _http, which owns the shared
/// consts and cannot: _http reexports this crate when the axum feature is on.
const H_SOCKET_ADDR: &str = "x-socket-addr";

/// Tower middleware inserting `x-socket-addr` from axum's real
/// `ConnectInfo<SocketAddr>` before the request reaches the graphql handler, so
/// `HttpIpSource::SocketAddr`, the default, has a framework provided source and
/// cannot be forgotten. Apply it with
/// `Router::layer(axum::middleware::from_fn(socket_addr_layer))` on a router
/// served through `into_make_service_with_connect_info::<SocketAddr>()`, which
/// is what puts the connect info into the request extensions.
///
/// The insert replaces any client supplied `x-socket-addr`: the header is only
/// trustworthy when it comes from the socket, letting the client win would
/// defeat the whole source. A request carrying no connect info, e.g. a test
/// driving the router without the make service, passes through untouched.
pub async fn socket_addr_layer(mut req: Request, next: Next) -> Response {
    if let Some(ConnectInfo(addr)) = req.extensions().get::<ConnectInfo<SocketAddr>>()
        && let Ok(v) = HeaderValue::from_str(&addr.to_string())
    {
        req.headers_mut().insert(H_SOCKET_ADDR, v);
    }
    next.run(req).await
}
