
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
use clap::{App, Arg};
mod helpers;
mod config;
mod commands;
pub mod results;


fn start() -> results::BearerResult<()> {


    let matches = App::new("bearer")
        .version("1.0")
        .author("Guillaume Gauvrit <guillaume@gauvr.it>")
        .about("Create Bearer Token from the command line")
        .arg(Arg::with_name("CONFIG")
            .short("c")
            .default_value("~/.config/bearer")
            .help("Set a custom config directory. Contains One file per client"))
        .arg(Arg::with_name("REGISTER")
            .short("r")
            .long("register")
            .help("Register a new client. This command is interactive."))
        .arg(Arg::with_name("CLIENT_NAME")
            .help("Set the client name")
            .required(true)
            .index(1))
        .get_matches();


    // Gets a value for config if supplied by user, or defaults to "default.conf"

    let config_dir = matches.value_of("CONFIG").unwrap();
    let client_name = matches.value_of("CLIENT_NAME".to_string()).unwrap();

    debug!("config_dir: {:?}", config_dir);
    debug!("client_name: {}", client_name);

    if matches.is_present("REGISTER") {
        commands::register::register(config_dir, client_name)?;
    } else {
        commands::display_header::display_header(config_dir, client_name)?;
    }
    Ok(())
}


fn main() {
    pretty_env_logger::init().unwrap();
    match start() {
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
