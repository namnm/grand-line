use crate::prelude::*;
use serde::de::DeserializeOwned;
use serde_json::{from_value, to_value};

/// Helper to quickly convert json.
pub trait JsonHelper
where
    Self: Sized + Serialize + DeserializeOwned,
{
    fn to_json(&self) -> Res<JsonValue> {
        let r = to_value(self)?;
        Ok(r)
    }
    fn from_json(v: JsonValue) -> Res<Self> {
        let r = from_value(v)?;
        Ok(r)
    }
}

/// Automatically implement for Serialize + `DeserializeOwned`.
impl<T> JsonHelper for T where T: Serialize + DeserializeOwned {}
