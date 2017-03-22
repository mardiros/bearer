//! # Bearer
//!
//! bearer is a command line utility to generate Authorization HTTP header
//! for bearer tokens. See https://tools.ietf.org/html/rfc6750
//!
//! bearer comes with a `--register` that will ask you the OAuth 2.0
//! client information to initialize the access and refresh tokens.
//!
//! Afterwhat it generate a header you can path to a curl request:
//!
//! ```
//! curl -H "$(bearer client_name)" "http://<oauth2 api>" | jq
//! ```
//!

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
