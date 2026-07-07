use crate::prelude::*;

/// Errors produced by the core utils helpers, currently json (de)serialization failures.
#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //

    // ========================================================================
    // server errors
    //
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
