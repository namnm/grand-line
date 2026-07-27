use subtle::ConstantTimeEq as _;

/// Compare a and b in constant time, avoiding a timing side channel when comparing secrets.
pub fn constant_time_eq(a: &str, b: &str) -> bool {
    a.as_bytes().ct_eq(b.as_bytes()).unwrap_u8() == 1
}
