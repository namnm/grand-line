use crate::prelude::*;

/// Errors produced by the file package, s3/r2 upload and processing chain.
#[grand_line_err]
pub enum MyErr {
    // ========================================================================
    // client errors
    //
    #[error("file {id} is not pending upload")]
    #[client]
    UploadNotPending {
        id: String,
    },

    // ========================================================================
    // server errors
    //
    #[error("s3 client is not configured on the graphql context")]
    S3ClientMissing,
    #[error("s3 operation failed: {inner}")]
    S3 {
        inner: String,
    },
    #[error("{program} failed: {inner}")]
    Process {
        program: String,
        inner: String,
    },
}

/// Wraps any displayable s3/presigning sdk error into MyErr::S3, for use with map_err.
pub fn err_s3<E>(e: E) -> GrandLineErr
where
    E: core::fmt::Display,
{
    MyErr::S3 {
        inner: e.to_string(),
    }
    .into()
}
