pub mod errors;
pub mod env;
mod iteratorext;
pub mod freedesktop;

use self::errors::*;

pub use std::result::Result as StdResult;

enum Io {
    Suppress,
    Inherit,
}

pub struct Options {
    io: Io,
}

pub trait Launch {
    fn launch(&self, args: Vec<String>) -> Error;
}
