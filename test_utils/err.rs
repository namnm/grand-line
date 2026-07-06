use crate::prelude::*;

#[grand_line_err]
pub enum TestErr {
    #[error("expect {message}")]
    Expect {
        message: String,
    },
}

impl TestErr {
    pub fn expect<T>(message: &str) -> Res<T> {
        let err = Self::Expect {
            message: message.into(),
        };
        Err(err.into())
    }
}
