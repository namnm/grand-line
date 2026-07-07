use crate::prelude::*;
use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD as B64};
use sha2::{Digest as _, Sha256};

/// Generate a random 32 byte secret, base64 encoded.
pub fn secret() -> String {
    b64_random(32)
}

/// Hash secret with SHA-256, base64 encoded, unsalted.
///
/// This is only ever called with the 256-bit, never with a low-entropy or
/// user-chosen value. A rainbow table over a 256-bit random input space is
/// not feasible, which is what salting exists to defend against, so no salt
/// is added here. Contrast with otp_hash, which salts because an otp is a
/// 6-digit number with only about 20 bits of entropy. Do not reuse
/// secret_hash for anything lower entropy than secret() without adding a
/// salt back in.
pub fn secret_hash(secret: &str) -> String {
    let b = Sha256::digest(secret.as_bytes());
    B64.encode(b)
}

/// Verify secret against a stored hash, using a constant time comparison.
pub fn secret_eq(secret_hashed: &str, secret: &str) -> bool {
    let secret_hashed2 = secret_hash(secret);
    constant_time_eq(secret_hashed, &secret_hashed2)
}
