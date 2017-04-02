#[macro_use]
extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate walkdir;
extern crate clap;
extern crate xdg;

use std::process::{Command, exit};
use std::os::unix::process::CommandExt;

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
    let matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .author(AUTHORS)
        .get_matches();

    let desktop_files = find_all_desktop_files().unwrap();
    let apps = Applications::new(desktop_files);
    apps.find_exact_match("", None);
}
