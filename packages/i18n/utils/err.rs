use _core::prelude::*;

#[grand_line_err]
pub enum MyErr {
    #[error("invalid locale: {locale}")]
    InvalidLocale {
        locale: String,
    },
    #[error("icu4x blob error: {inner}")]
    IcuBlob {
        inner: String,
    },
    #[error("icu4x init error: {inner}")]
    IcuInit {
        inner: String,
    },
    #[error("invalid timestamp: {value}")]
    InvalidTimestamp {
        value: i64,
    },
    #[error("missing variable: {name}")]
    MissingVar {
        name: String,
    },
    #[error("invalid value for variable {name}: expected a number, actual: {value}")]
    InvalidNumber {
        name: String,
        value: String,
    },
}
