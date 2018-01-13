pub mod errors;
pub mod env;
mod freedesktop;
mod exec;
mod keyword;

pub use self::errors::*;

pub use std::result::Result as StdResult;

mod search;
pub use self::search::{Search, SearchTerms};

mod launch;
pub use self::launch::Launch;

pub use self::env::Lunchable;

enum Io {
    Suppress,
    Inherit,
}

pub struct Options {
    io: Io,
}
