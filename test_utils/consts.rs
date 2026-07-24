#[cfg(feature = "authz")]
pub use _authz::consts::*;
#[cfg(feature = "http")]
pub use _http::consts::*;

#[cfg(feature = "http")]
pub const UA: &str = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/142.0.0.0 Safari/537.36";
#[cfg(feature = "http")]
pub const UA_SEC_CH: &str = r#""Chromium";v="142", "Google Chrome";v="142", "Not_A Brand";v="99""#;
