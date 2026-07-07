use crate::prelude::*;
use serde_qs::{from_str, to_string};

/// An id/secret pair serialized as a query string token.
#[derive(Serialize, Deserialize)]
pub struct QsToken {
    pub id: String,
    pub secret: String,
}

/// Serialize id and secret into a query string token.
pub fn qs_token(id: &str, secret: &str) -> Res<String> {
    let t = to_string(&QsToken {
        id: id.to_owned(),
        secret: secret.to_owned(),
    })
    .map_err(MyErr::from)?;
    Ok(t)
}

/// Parse a query string token back into a QsToken, None if empty or invalid.
pub fn qs_token_parse(token: &str) -> Option<QsToken> {
    if token.is_empty() {
        None
    } else {
        from_str::<QsToken>(token).ok()
    }
}
