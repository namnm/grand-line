use crate::prelude::*;

/// Provides a debug-context prefix and span for building syn errors.
pub trait AttrDebug {
    /// Human readable context prefix used in error messages, e.g. the
    /// attribute or field name being validated.
    fn attr_debug(&self) -> String;
    /// Span used for constructed errors, defaults to the call site.
    fn span(&self) -> Span {
        Span::call_site()
    }
    /// Builds a syn error combining attr_debug() and msg, skipping empty parts.
    fn syn_err(&self, msg: &str) -> SynErr {
        let msg = [self.attr_debug(), msg.to_owned()]
            .into_iter()
            .filter(|v| !v.is_empty())
            .collect::<Vec<_>>()
            .join(" ");
        SynErr::new(self.span(), msg)
    }
}
