#![feature(try_from)]
#![feature(slice_patterns)]

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

use std::convert::TryInto;

use clap::App;

mod lunch;

use lunch::*;
use lunch::errors::*;

const AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const DESCRIPTION: &str = env!("CARGO_PKG_DESCRIPTION");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    env_logger::init().chain_err(
        || "Error initialising logging",
    )?;
    let locale = lunch::freedesktop::locale::get_locale()?;
    let arg_matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .author(AUTHORS)
        .get_matches();
    //    arg_matches.

    let term = "";
    let apps = lunch::freedesktop::find_all_desktop_files()?;
    return apps.find_exact_match(term, &locale)
        .chain_err(|| format!("Error finding match for '{}'", term))
        .map(|entry| {
            debug!("Found match: {:?}", entry);
            use lunch::freedesktop::entry::*;
            let name = entry.name.clone();
            let exec: Result<ApplicationEntry> = entry.try_into();
            match exec {
                Err(err) => {
                    error!("Error launching entry named '{}': {}", name, err);
                    return Err(err);
                }
                Ok(exec) => {
                    let err = exec.launch(vec![]);
                    error!("Error launching entry named '{}': {}", name, err);
                    return Err(err);
                }
            }
        })?;
}
