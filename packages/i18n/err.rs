use _core::prelude::*;

#[grand_line_err]
pub enum MyErr {
    #[error("invalid locale: {0}")]
    InvalidLocale(String),
    #[error("icu4x blob error: {0}")]
    IcuBlob(String),
    #[error("icu4x init error: {0}")]
    IcuInit(String),
}
