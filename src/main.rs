
#[macro_use]
extern crate log;
extern crate pretty_env_logger;

extern crate clap;
extern crate libc;
extern crate tilde_expand;
extern crate toml;
extern crate chrono;

extern crate serde;
#[macro_use]
extern crate serde_derive;

extern crate url;
extern crate curl;
extern crate serde_json;

use std::io::Write;
mod helpers;
mod config;
mod commands;
mod results;


fn main() {
    pretty_env_logger::init().unwrap();
    match commands::start() {
        Ok(()) => {
            debug!("Command bearer ended successfully");
        }
        Err(results::BearerError::ValueError(msg)) => {
            let _ = writeln!(&mut std::io::stderr(), "ERROR: {}", msg);
            std::process::exit(1);
        }
        Err(results::BearerError::IOError(err)) => {
            let _ = writeln!(&mut std::io::stderr(), "ERROR: {}", err);
            std::process::exit(1);
        }
        Err(err) => {
            let _ = writeln!(&mut std::io::stderr(), "ERROR: {:?}", err);
            std::process::exit(101);
        }
    }
}
