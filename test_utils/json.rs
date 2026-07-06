use crate::prelude::*;

pub trait JsonTestingHelper
where
    Self: Default,
{
    fn ptr<'a>(&'a self, path: &str) -> &'a Self;
    fn str<'a>(&'a self, path: &str) -> &'a str;
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
