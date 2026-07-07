use crate::prelude::*;
use argon2::{
    Argon2, PasswordHasher as _, PasswordVerifier as _,
    password_hash::{PasswordHash, SaltString},
};
use rand::{Rng as _, rng};
use zxcvbn::{Score, zxcvbn};

/// Reject passwords weaker than zxcvbn score 3, out of 0-4.
pub fn password_validate(password: &str) -> Res<()> {
    if zxcvbn(password, &[]).score() < Score::Three {
        return Err(MyErr::PasswordInvalid.into());
    }
    Ok(())
}

/// Hash password with Argon2, using a fresh random salt.
pub fn password_hash(password: &str) -> Res<String> {
    let mut b = [0u8; 16];
    rng().fill_bytes(&mut b);
    let salt = SaltString::encode_b64(&b).map_err(MyErr::from)?;
    let password_hashed = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(MyErr::from)?
        .to_string();
    Ok(password_hashed)
}

/// Verify password against a stored Argon2 hash, returns false if the hash is malformed.
pub fn password_eq(password_hashed: &str, password: &str) -> bool {
    match PasswordHash::new(password_hashed) {
        Ok(v) => Argon2::default().verify_password(password.as_bytes(), &v).is_ok(),
        _ => false,
    }
}
