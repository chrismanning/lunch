#![feature(try_from)]

extern crate lunch;

extern crate error_chain;
#[macro_use]
extern crate log;
extern crate env_logger;
extern crate clap;

use std::convert::TryInto;

use clap::App;

use lunch::*;
use lunch::errors::*;
use lunch::freedesktop::locale::Locale;

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
    let locale = Locale::from_env()?;
    let arg_matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .author(AUTHORS)
        .get_matches();
    //    arg_matches.

    let term = "";
//    let apps = lunch::freedesktop::find_all_desktop_files()?;
//    apps.find_exact_match(term, &locale)
//        .chain_err(|| format!("Error finding match for '{}'", term))
//        .map(|entry| {
//            debug!("Found match: {:?}", entry);
//            use lunch::freedesktop::entry::*;
//            let name = entry.name.clone();
//            let exec: Result<ApplicationEntry> = entry.try_into();
//            match exec {
//                Err(err) => {
//                    error!("Error launching entry named '{}': {}", name, err);
//                    Err(err)
//                }
//                Ok(exec) => {
//                    let err = exec.launch(vec![]);
//                    error!("Error launching entry named '{}': {}", name, err);
//                    Err(err)
//                }
//            }
//        })?
    unimplemented!()
}
