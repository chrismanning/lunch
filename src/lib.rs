#![recursion_limit = "128"]
#![feature(try_from)]
#![feature(slice_patterns)]

extern crate clap;
#[macro_use]
extern crate derive_builder;
extern crate env_logger;
#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate peeking_take_while;
extern crate xdg;

#[cfg(test)]
extern crate spectral;
#[cfg(test)]
extern crate tempdir;

#[macro_use]
extern crate maplit;

mod lunch;
pub use lunch::*;
