#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

#[cfg(feature = "axum")]
mod axum;
mod consts;
mod db;
mod err;
mod exec_assert;
mod schema;

pub mod prelude {
    #[cfg(feature = "axum")]
    pub use super::axum::*;
    pub use super::consts::*;
    pub use super::db::*;
    pub use super::err::*;
    pub use super::exec_assert::*;
    pub use super::schema::*;
    pub use _utils::*;
    pub use pretty_assertions::assert_eq as pretty_eq;
}
