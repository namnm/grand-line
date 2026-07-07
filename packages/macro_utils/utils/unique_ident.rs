use crate::prelude::*;
use ulid::Ulid;

/// Generates a hygienic, collision-free identifier (as a token stream) for use
/// in macro-generated code, prefixed with __grandline_ and backed by a ULID.
pub fn unique_ident() -> Ts2 {
    let id = Ulid::new().to_string().to_lowercase();
    let tmp = format!("__grandline_{id}");
    Ident::new(&tmp, Span::mixed_site()).to_token_stream()
}
