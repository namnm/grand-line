use crate::prelude::*;

/// Parses a token stream as a single named struct field.
pub trait Ts2ToField {
    /// Parses self as a named field (name colon type), erroring if it is not one.
    fn field_or_err(self) -> SynRes<Field>;
}

impl Ts2ToField for Ts2 {
    fn field_or_err(self) -> SynRes<Field> {
        Parser::parse2(Field::parse_named, self)
    }
}
