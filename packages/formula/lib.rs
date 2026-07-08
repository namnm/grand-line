#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod accessor;
mod cache;
mod context;
mod dep_graph;
mod engine;
mod err;
mod eval;
mod opts;
mod preprocess;
mod resolver;

pub mod export {
    pub use crate::accessor::*;
    pub use crate::context::*;
    pub use crate::dep_graph::*;
    pub use crate::err::*;
    pub use crate::eval::*;
    pub use crate::opts::*;
    pub use crate::preprocess::*;
    pub use crate::resolver::*;
}

pub mod reexport {
    pub use rhai;
    pub use sourcemap;
}

pub mod prelude {
    pub use crate::export::*;
    pub use crate::reexport::*;
    pub use rhai::{Dynamic as FormulaDynamic, Map as FormulaMap};
    pub use sourcemap::SourceMap as FormulaSourceMap;

    pub(crate) use crate::err::FormulaErr as MyErr;
    pub(crate) use _core::prelude::*;
}
