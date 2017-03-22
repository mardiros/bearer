use std::io;
use std::io::prelude::*;

use super::super::config::Config;
use super::super::helpers::path::build_path;
use super::super::helpers::oauth2;
use super::super::results::{BearerResult, BearerError};

fn read_stdin(message: &str) -> BearerResult<String> {
    print!("{}", message);
    io::stdout().flush().unwrap();

    let mut buffer = String::new();
    match io::stdin().read_line(&mut buffer) {
        Ok(_) => Ok(buffer.trim().to_string()),
        Err(err) => Err(BearerError::IOError(format!("{}", err))),
    }
}


pub fn register(config_dir: &str, client_name: &str) -> BearerResult<()> {

    debug!("Registering new client {} in directory {}",
           client_name,
           config_dir);
    let (_, exists) = build_path(config_dir, client_name)?;
    if exists {
        return Err(BearerError::ValueError(format!("Client {} already registered", client_name)));
    }
    println!("Before continue, register the a client with the following url to the OAuth2 \
              Provider:");
    println!("");
    println!("http://localhost:6750/callback");
    println!("");
    println!("Ensure your port is not already open by another service.");
    println!("If the provider require a https url, please run an https reverse proxy before \
              continue.");
    println!("");
    let server_url = read_stdin("Enter the OAuth2.0 Server Url: ")?;
    let client_id = read_stdin("Enter the Client Id: ")?;
    let secret = read_stdin("Enter the Client Secret: ")?;

    let mut config = Config::new(config_dir,
                                 client_name,
                                 server_url.as_str(),
                                 client_id.as_str(),
                                 secret.as_str())?;

    debug!("Start server to retrieve tokens");
    let tokens = oauth2::get_tokens(&config)?;
    debug!("Token retrieved: {:?}", tokens);
    config.set_tokens(tokens);
    config.write()
}
