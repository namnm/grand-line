use crate::prelude::*;

/// Re-exported so an app can set HttpConfig::cookie_same_site without depending
/// on the cookie crate itself.
pub use cookie::SameSite;

/// Where get_ip reads the client address from.
/// A header is only trustworthy behind a proxy that sets it, reachable directly
/// any client picks its own address, so the source has to be declared.
#[derive(Clone)]
pub enum HttpIpSource {
    /// The x-socket-addr header the app sets from the real connection.
    /// The only source no client can forge, and the default.
    SocketAddr,
    /// A header a trusted proxy in front of this app sets.
    /// hops is how many entries from the right of a comma separated list the
    /// trusted proxies contribute, 1 for a single proxy appending the address it
    /// saw, 2 when another trusted proxy sits behind it.
    Proxy {
        header: &'static str,
        hops: usize,
    },
}

/// Runtime configuration for the http package, request ip source and cookie attributes.
#[derive(Clone)]
pub struct HttpConfig {
    pub ip_source: HttpIpSource,
    /// SameSite on every cookie set_cookie writes.
    /// Lax is the browser default and the main csrf control for a cookie
    /// authenticated api, a browser app on a different origin than the api needs
    /// None instead, which browsers only accept together with Secure.
    pub cookie_same_site: SameSite,
    /// Path on every cookie set_cookie writes.
    /// Without it the browser derives one from the request uri, so a cookie set
    /// from /api/graphql is never sent to any other path.
    pub cookie_path: &'static str,
    /// Whether every cookie set_cookie writes carries Secure.
    /// Browsers drop a Secure cookie over plain http, so cookie based login
    /// silently never works in local development. Only turn this off for
    /// deployments that are genuinely plain http end to end, e.g. tests or an
    /// http-only local environment behind no proxy.
    pub cookie_secure: bool,
}

impl Default for HttpConfig {
    fn default() -> Self {
        Self {
            ip_source: HttpIpSource::SocketAddr,
            cookie_same_site: SameSite::Lax,
            cookie_path: "/",
            cookie_secure: true,
        }
    }
}
