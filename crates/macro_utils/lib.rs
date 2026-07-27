#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod macros;

mod attr;
mod utils;
pub use attr::*;
pub use utils::*;

pub use heck;
pub use maplit;
pub use proc_macro2;
pub use quote;
pub use serde;
pub use strum;
pub use strum_macros;
pub use syn;

mod prelude {
    pub use crate::*;
    use_common_macro_utils!();
    use_common_std!();
}
