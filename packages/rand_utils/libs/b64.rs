use crate::prelude::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use rand::{Rng as _, rng};

/// Generate bytes of random data and base64 encode it, URL-safe without padding.
pub fn b64_random(bytes: usize) -> String {
    let mut b = vec![0u8; bytes];
    rng().fill_bytes(&mut b);
    B64.encode(b)
}

/// Base64 encode s, URL-safe without padding.
pub fn b64_encode(s: &str) -> String {
    B64.encode(s.as_bytes())
}
