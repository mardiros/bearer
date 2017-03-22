use clap::{App, Arg};

use super::results;

mod register;
mod display_header;


pub fn start() -> results::BearerResult<()> {


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
        register::register(config_dir, client_name)?;
    } else {
        display_header::display_header(config_dir, client_name)?;
    }
    Ok(())
}
