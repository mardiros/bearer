use clap::{App, Arg};

use super::results;

mod register;
mod display_header;
mod list;
mod refresh;


pub fn start() -> results::BearerResult<()> {


    let matches = App::new("bearer")
        .version("0.2.0")
        .author("Guillaume Gauvrit <guillaume@gauvr.it>")
        .about("Create Bearer Token from the command line")
        .arg(Arg::with_name("CONFIG")
            .short("c")
            .default_value("~/.config/bearer")
            .help("Set a custom config directory. Contains One file per client."))
        .arg(Arg::with_name("LIST")
            .short("l")
            .long("list")
            .conflicts_with("REGISTER")
            .help("List registered clients."))
        .arg(Arg::with_name("REGISTER")
            .short("r")
            .long("register")
            .conflicts_with("LIST")
            .help("Register a new client. This command is interactive."))
        .arg(Arg::with_name("REFRESH")
            .short("u")
            .long("refresh")
            .conflicts_with("LIST")
            .help("Refresh an existing client. This command is interactive."))
        .arg(Arg::with_name("CLIENT_NAME")
            .help("Set the client name.")
            .required_unless_one(&["LIST"])
            .index(1))
        .get_matches();


    // Gets a value for config if supplied by user, or defaults to "default.conf"

    let config_dir = matches.value_of("CONFIG").unwrap();
    let client_name = matches.value_of("CLIENT_NAME".to_string());

    debug!("config_dir: {:?}", config_dir);
    debug!("client_name: {:?}", client_name);

    if matches.is_present("LIST") {
        list::command(config_dir)?;
    } else if matches.is_present("REGISTER") {
        register::command(config_dir, client_name.unwrap())?;
    } else if matches.is_present("REFRESH") {
        refresh::command(config_dir, client_name.unwrap())?;
    } else {
        display_header::command(config_dir, client_name.unwrap())?;
    }
    Ok(())
}
