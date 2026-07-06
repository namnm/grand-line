use crate::prelude::*;
use serde_json::{from_value, to_value};

/// Helper to quickly serialize to json.
pub trait ToJson
where
    Self: Serialize,
{
    fn to_json(&self) -> Res<JsonValue> {
        let r = to_value(self)?;
        Ok(r)
    }
}

/// Automatically implement for Serialize.
impl<T> ToJson for T where T: Serialize
{
}

/// Helper to quickly deserialize from json.
pub trait FromJson
where
    Self: Sized + DeserializeOwned,
{
    fn from_json(v: JsonValue) -> Res<Self> {
        let r = from_value(v)?;
        Ok(r)
    }
}

/// Automatically implement for DeserializeOwned.
impl<T> FromJson for T where T: DeserializeOwned
{
}
