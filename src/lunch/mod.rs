use std::ffi::OsStr;

pub mod errors;
mod iteratorext;
pub mod freedesktop;

use errors::*;

pub use std::result::Result as StdResult;

pub trait Exec {
    fn exec<I, S>(&self, args: I) -> Error
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>;
}
