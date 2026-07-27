use crate::prelude::*;

/// Errors surfaced by the core utils helper package, split into client-facing and server-only variants.
#[grand_line_err]
pub enum MyErr {
    // ------------------------------------------------------------------------
    // client errors
    // ------------------------------------------------------------------------

    // ------------------------------------------------------------------------
    // server errors
    // ------------------------------------------------------------------------
    #[error("json error: {inner}")]
    Json {
        #[from]
        inner: JsonErr,
    },
    #[error("not implemented")]
    NotImpl,
}

impl From<JsonErr> for GrandLineErr {
    fn from(v: JsonErr) -> Self {
        MyErr::from(v).into()
    }
}
