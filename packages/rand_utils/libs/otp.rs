use crate::prelude::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use hmac::{Hmac, KeyInit as _, Mac as _};
use rand::{RngExt as _, rng};
use sha2::Sha256;

/// Generate a random 6 digit numeric OTP, zero-padded.
pub fn otp() -> String {
    let otp = rng().random_range(0..=999_999);
    format!("{otp:06}")
}

/// Hash otp with a fresh random salt, returns (salt, hashed otp) to be stored.
pub fn otp_hash(otp: &str) -> Res<(String, String)> {
    let salt = b64_random(8);
    let otp_hashed = otp_hash_with_salt(&salt, otp)?;
    Ok((salt, otp_hashed))
}

/// Verify otp against a stored salt and hash, using a constant time comparison.
pub fn otp_eq(salt: &str, otp_hashed: &str, otp: &str) -> Res<bool> {
    let otp_hashed2 = otp_hash_with_salt(salt, otp)?;
    let r = constant_time_eq(otp_hashed, &otp_hashed2);
    Ok(r)
}

/// Compute the HMAC-SHA256 of otp keyed by salt, base64 encoded.
pub fn otp_hash_with_salt(salt: &str, otp: &str) -> Res<String> {
    let mut mac = Hmac::<Sha256>::new_from_slice(salt.as_bytes()).map_err(|e| MyErr::HmacErr {
        inner: e.to_string(),
    })?;
    mac.update(otp.as_bytes());
    let b = mac.finalize().into_bytes();
    let secret = B64.encode(b);
    Ok(secret)
}
