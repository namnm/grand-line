use super::prelude::*;

#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //
    #[error("internal server error")]
    #[client]
    InternalServer,

    // ========================================================================
    // server errors
    //
    #[error("context missing grand line context")]
    Ctx404,
    #[error("context missing sea orm database")]
    CtxDb404,

    #[error("commit error: transaction is still in use elsewhere")]
    TxCommit,
    #[error("rollback error: transaction is still in use elsewhere")]
    TxRollback,

    #[error("data loader cannot downcast from arc dyn any")]
    LoaderDowncast,
    #[error("data loader cannot get string key with column {col}")]
    LoaderKeyNone {
        col: String,
    },

    #[error("cache cannot downcast from once cell arc")]
    CacheDowncast,
}

impl From<MyErr> for ServerError {
    fn from(v: MyErr) -> Self {
        GrandLineErr(Arc::new(v)).into()
    }
}
