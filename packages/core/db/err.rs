use super::prelude::*;

#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //
    #[error("data not found")]
    #[client]
    Db404,

    // ========================================================================
    // server errors
    //
    #[error("database error: {inner}")]
    Db {
        #[from]
        inner: DbErr,
    },
    #[error("{col} column not found")]
    DbCol404 {
        col: String,
    },
    #[error("id not set")]
    IdNotSet,

    #[error("resolver try to unwrap with no value")]
    GqlResolverNone,
    #[error("look ahead selection fields len should be 1")]
    GqlLookAhead,
}

impl From<DbErr> for GrandLineErr {
    fn from(v: DbErr) -> Self {
        MyErr::from(v).into()
    }
}
