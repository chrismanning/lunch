#![feature(conservative_impl_trait)]

#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate walkdir;
extern crate clap;
extern crate xdg;
#[macro_use]
extern crate derive_builder;

use clap::*;
use error_chain::*;

mod desktop;

use desktop::*;

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    env_logger::init().unwrap();
    let arg_matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .author(AUTHORS)
        .get_matches();
//    arg_matches.

    let term = "";
    let apps = find_all_desktop_files().unwrap();
    match apps.find_exact_match(term, None) {
        Ok(entry) => {
            debug!("Found match: {:?}", entry);
            let err = entry.launch();
            error!("Error launching entry named '{}': {}", entry.name, err);
        },
        Err(err) => {
            error!("Error finding match for '{}': {}", term, err);
        }
    }
}
