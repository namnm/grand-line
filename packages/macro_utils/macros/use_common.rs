/// Internal macro to reuse common imports.
#[macro_export]
macro_rules! use_common_std {
    () => {
        pub use core::any::{Any, TypeId};
        pub use core::error::Error;
        pub use core::fmt::{Debug, Display, Error as FmtErr, Formatter, Result as FmtRes};
        pub use core::future::Future;
        pub use core::hash::{Hash, Hasher};
        pub use core::marker::PhantomData;
        pub use core::str::FromStr;
        pub use std::collections::{HashMap, HashSet};
        pub use std::sync::{Arc, LazyLock};
        pub type ArcAny = Arc<dyn Any + Send + Sync>;
    };
}

#[macro_export]
macro_rules! use_common_macro_utils {
    () => {
        pub use heck::*;
        pub use maplit::*;
        pub use proc_macro2::{Span, TokenStream as Ts2};
        pub use quote::*;
        pub use syn::{Error as SynErr, Result as SynRes, parse::*, punctuated::*, spanned::Spanned, token::Comma, *};
    };
}
