#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate walkdir;
extern crate clap;
use clap::*;

mod desktop;

use desktop::*;
use desktop::locale::Locale;

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .author(AUTHORS)
        .get_matches();

    let x = find_exact_match("test", None);
}
