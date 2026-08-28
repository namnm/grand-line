#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod db;
mod graphql;
mod graphql_types;
mod subscription;
mod utils;

pub mod export {
    pub use crate::db::*;
    pub use crate::graphql::*;
    pub use crate::graphql_types::*;
    pub use crate::subscription::*;
    pub use crate::utils::*;
    pub use _proc::*;
    pub use _utils_proc::{PartialEqString, field_names};
}

pub mod reexport {
    pub use _utils::{maplit, strum, strum_macros};
    pub use async_graphql;
    pub use async_trait;
    pub use chrono;
    pub use sea_orm;
    pub use serde;
    pub use serde_json;
    pub use serde_with;
    pub use sqlx;
    pub use thiserror;
    pub use tokio;
    pub use ulid;
}

pub mod prelude {
    pub use crate::export::*;
    pub use crate::reexport::*;
    pub use async_graphql::{
        Error as GraphQLErr, MaybeUndefined as Undefined, Result as GraphQLRes, Schema as GraphQLSchema,
        Value as GraphQLValue,
        extensions::*,
        futures_util::{Stream, StreamExt, future::BoxFuture},
        *,
    };
    pub use async_trait::async_trait;
    pub use sea_orm::{
        DbErr, Schema as DbSchema, Value as DbValue,
        entity::prelude::*,
        prelude::*,
        sea_query::{IntoCondition, SimpleExpr},
        *,
    };
    pub use serde::{
        de::DeserializeOwned,
        {Deserialize, Serialize},
    };
    pub use serde_json::{Error as JsonErr, from_str as json_parse, json, to_string as json_string};
    pub use thiserror::Error as ThisErr;
    pub use tokio::{
        join,
        sync::{Mutex, OnceCell},
        task::{spawn, spawn_blocking, spawn_local},
        time::{interval, sleep, timeout},
        try_join,
    };
    _utils::use_common_std!();
    // override
    pub use crate::export::model;
    pub use async_graphql::SchemaBuilder;
}
