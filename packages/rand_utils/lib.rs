#![allow(ambiguous_glob_reexports, dead_code, unused_imports)]

mod b64;
mod eq;
mod err;
mod otp;
mod password;
mod qs;
mod secret;

pub mod export {
    pub use crate::err::MyErr as AuthUtilsErr;
    pub mod rand_utils {
        pub use crate::b64::*;
        pub use crate::eq::*;
        pub use crate::otp::*;
        pub use crate::password::*;
        pub use crate::qs::*;
        pub use crate::secret::*;
    }
}

pub mod reexport {
    pub use argon2;
    pub use base64;
    pub use hmac;
    pub use rand;
    pub use rand_core;
    pub use serde_qs;
    pub use sha2;
    pub use subtle;
    pub use zxcvbn;
}

pub mod prelude {
    pub use crate::export::*;
    pub use crate::reexport::*;

    pub(crate) use crate::export::AuthUtilsErr as MyErr;
    pub(crate) use crate::export::rand_utils::*;
    pub(crate) use _core::prelude::*;
}
