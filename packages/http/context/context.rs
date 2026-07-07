use crate::prelude::*;
use cookie::{
    Cookie,
    time::{Duration, OffsetDateTime},
};
use core::net::{IpAddr, SocketAddr};

/// Read-only access to the current request's headers, IP, user agent, and cookies.
pub trait HttpContext<'a>
where
    Self: ImplContext<'a>,
{
    /// Extract user-agent related headers (user-agent and sec-ch-ua-*) from h.
    fn get_ua_raw(h: Option<HashMap<String, Vec<String>>>) -> Res<HashMap<String, String>> {
        let mut m = HashMap::<String, String>::new();
        for (k, v) in &h.ok_or(MyErr::CtxHeaders404)? {
            let k = k.as_str();
            if k.starts_with(H_UA_SEC_CH) || k == H_UA {
                if v.len() > 1 {
                    return Err(MyErr::HeaderMultipleValues {
                        k: k.to_owned(),
                    }
                    .into());
                }
                m.insert(k.to_owned(), v.first().cloned().unwrap_or_default());
            }
        }
        Ok(m)
    }

    // Will be overridden by the implementation below.
    fn try_headers(&self) -> Res<Option<HashMap<String, Vec<String>>>> {
        Err(MyErr::MissingImplementation.into())
    }

    /// Read header k, empty string if absent, Err if it has more than one value.
    fn get_header(&self, k: &str) -> Res<String> {
        let req_headers = self.try_headers()?.ok_or(MyErr::CtxHeaders404)?;
        let Some(v) = req_headers.get(k) else {
            return Ok(String::new());
        };
        if v.len() > 1 {
            return Err(MyErr::HeaderMultipleValues {
                k: k.to_owned(),
            }
            .into());
        }
        let v = v.first().cloned().unwrap_or_default();
        Ok(v)
    }

    /// Resolve the client IP, trying x-real-ip, then x-forwarded-for, then
    /// x-socket-addr, taking the first entry of a comma separated list.
    fn get_ip(&self) -> Res<String> {
        let mut v = self.get_header(H_REAL_IP)?;
        if v.is_empty() {
            v = self.get_header(H_FORWARDED_FOR)?;
        }
        if v.is_empty() {
            v = self.get_header(H_SOCKET_ADDR)?;
        }
        let raw = v.split(',').next().unwrap_or_default().trim();
        let ip = if let Ok(sa) = raw.parse::<SocketAddr>() {
            sa.ip().to_string()
        } else {
            raw.to_owned()
        };
        if IpAddr::from_str(&ip).is_err() {
            return Err(MyErr::HeaderIp404.into());
        }
        Ok(ip)
    }

    /// Read the user-agent related headers, Err if the user-agent header is absent.
    fn get_ua(&self) -> Res<HashMap<String, String>> {
        if self.get_header(H_UA)?.is_empty() {
            return Err(MyErr::HeaderUa404.into());
        }
        let h = self.try_headers()?;
        let ua = Self::get_ua_raw(h)?;
        Ok(ua)
    }

    /// Read the Authorization header value, with a leading "Bearer " prefix stripped if present.
    fn get_authorization_token(&self) -> Res<String> {
        let h = self.get_header(H_AUTHORIZATION)?;
        Ok(h.strip_prefix(BEARER).unwrap_or(&h).to_owned())
    }

    /// Parse the Cookie header into a name to value map, skipping unparsable entries.
    fn get_cookies(&self) -> Res<HashMap<String, String>> {
        let h = self.get_header(H_COOKIE)?;
        let mut m = HashMap::new();
        for c in h.split(';') {
            if let Ok(kv) = Cookie::parse(c) {
                m.insert(kv.name().to_owned(), kv.value().to_owned());
            }
        }
        Ok(m)
    }

    /// Read a single cookie by name.
    fn get_cookie(&self, k: &str) -> Res<Option<String>> {
        let v = self.get_cookies()?.get(k).cloned();
        Ok(v)
    }

    /// Append a Set-Cookie response header, http-only and secure, expiring
    /// expires milliseconds from now.
    fn set_cookie(&self, k: &str, v: &str, expires: i64) {
        let v = Cookie::build(Cookie::new(k, v))
            .http_only(true)
            .secure(true)
            .max_age(Duration::seconds(expires / 1000))
            .expires(OffsetDateTime::now_utc() + Duration::milliseconds(expires))
            .build()
            .to_string();
        self.append_http_header_impl(H_SET_COOKIE, &v);
    }
}

impl<'a> HttpContext<'a> for Context<'a> {
    #[cfg(feature = "axum")]
    fn try_headers(&self) -> Res<Option<HashMap<String, Vec<String>>>> {
        Ok(self.get_headers())
    }
}
