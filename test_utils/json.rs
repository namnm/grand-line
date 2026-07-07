use crate::prelude::*;

/// JSON pointer lookups for tests, missing or mistyped paths fall back to
/// a default value instead of panicking or returning an Option.
pub trait JsonTestingHelper
where
    Self: Default,
{
    /// Look up path, returns the default value when the path is missing.
    fn ptr<'a>(&'a self, path: &str) -> &'a Self;
    /// Look up path as a string, returns an empty string on a missing or non-string value.
    fn str<'a>(&'a self, path: &str) -> &'a str;
    /// Look up path as an array, returns an empty array on a missing or non-array value.
    fn arr<'a>(&'a self, path: &str) -> &'a Vec<Self>;
}

impl JsonTestingHelper for JsonValue {
    fn ptr<'a>(&'a self, path: &str) -> &'a Self {
        self.pointer(path).unwrap_or_default()
    }
    fn str<'a>(&'a self, path: &str) -> &'a str {
        self.ptr(path).as_str().unwrap_or_default()
    }
    fn arr<'a>(&'a self, path: &str) -> &'a Vec<Self> {
        static EMPTY: Vec<JsonValue> = Vec::new();
        self.ptr(path).as_array().unwrap_or(&EMPTY)
    }
}
