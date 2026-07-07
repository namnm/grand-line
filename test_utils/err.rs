use crate::prelude::*;

#[grand_line_err]
pub enum TestErr {
    #[error("expect {message}")]
    Expect {
        message: String,
    },
}

impl TestErr {
    /// Build an Err carrying message, useful as an early return for a failed assertion.
    pub fn expect<T>(message: &str) -> Res<T> {
        let err = Self::Expect {
            message: message.into(),
        };
        Err(err.into())
    }
}
