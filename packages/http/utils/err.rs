use crate::prelude::*;

#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //
    #[error("request header {k} has more than 1 value")]
    #[client]
    HeaderMultipleValues {
        k: String,
    },
    #[error("ip address is missing in the request headers")]
    #[client]
    HeaderIp404,
    #[error("user agent is missing in the request headers")]
    #[client]
    HeaderUa404,

    // ========================================================================
    // server errors
    //
    #[error("http implementation is not configured")]
    MissingImplementation,
    #[error("request headers are missing in the context")]
    CtxHeaders404,
}
