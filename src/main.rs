extern crate lunch;

extern crate clap;
extern crate env_logger;
extern crate error_chain;
#[macro_use]
extern crate log;

use clap::{App, Arg};

use log::LogLevelFilter;
use env_logger::LogBuilder;

use lunch::errors::*;
use lunch::env::LunchEnv;

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
    let mut log_builder = LogBuilder::new();

    let arg_matches = App::new(APP_NAME)
        .version(VERSION)
        .about(DESCRIPTION)
        .author(AUTHORS)
        .arg(Arg::with_name("keyword")
            .short("k")
            .long("keyword")
            .conflicts_with("terms")
            .value_name("KEYWORD")
            .takes_value(true)
            .help("Search by keyword")
        )
        .arg(Arg::with_name("terms")
            .value_name("TERMS")
            .help("General search terms")
            .conflicts_with("keyword")
            .multiple(true)
            .required(true)
        )
        .arg(Arg::with_name("debug")
            .short("d")
            .long("debug")
            .help("Enable debug logging output")
        )
        .arg(Arg::with_name("trace")
            .short("t")
            .long("trace")
            .help("Enable trace logging output")
        )
        .get_matches();

    if arg_matches.is_present("debug") {
        log_builder.filter(None, LogLevelFilter::Debug);
    }
    if arg_matches.is_present("trace") {
        log_builder.filter(None, LogLevelFilter::Trace);
    }

    log_builder.init().chain_err(
        || "Error initialising logging",
    )?;

    let env = LunchEnv::init()?;

    if let Some(keyword) = arg_matches.value_of("keyword") {
        info!("Searching for keyword '{}'", keyword);
        if let Some(lunchable) = env.keyword(keyword) {
            return Err(lunchable.launch(vec![]));
        }
    }

    unimplemented!()
}
