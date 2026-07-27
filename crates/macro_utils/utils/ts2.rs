use crate::prelude::*;

/// Parses a string into a proc_macro2 token stream.
pub trait StringToTs2 {
    /// Parses self as Rust tokens, returning a syn error on invalid syntax.
    fn ts2_or_err(&self) -> SynRes<Ts2>;
}

impl StringToTs2 for String {
    fn ts2_or_err(&self) -> SynRes<Ts2> {
        self.parse::<Ts2>()
            .map_err(|e| SynErr::new(Span::call_site(), e.to_string()))
    }
}

impl StringToTs2 for str {
    fn ts2_or_err(&self) -> SynRes<Ts2> {
        self.to_owned().ts2_or_err()
    }
}
