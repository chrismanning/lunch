#![feature(try_from)]
#![feature(slice_patterns)]
#![feature(advanced_slice_patterns)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;
extern crate xdg;
#[macro_use]
extern crate derive_builder;
#[macro_use]
extern crate maplit;
#[macro_use]
extern crate lazy_static;
extern crate regex;

use std::ffi::OsStr;

mod lunch;
pub use lunch::*;
