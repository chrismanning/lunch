use std::ffi::OsStr;

pub mod errors;
pub mod env;
mod iteratorext;
pub mod freedesktop;

use errors::*;

pub use std::result::Result as StdResult;

enum Io {
    Suppress,
    Inherit,
}

pub struct Options {
    io: Io,
}

pub trait Application {}

pub trait ApplicationIndex {}

pub trait Launch: Application + ApplicationIndex {
    fn launch(&self, args: Vec<String>) -> Error;
}
