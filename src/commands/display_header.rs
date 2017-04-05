use std::io;
use std::io::prelude::*;

use super::super::config::Config;
use super::super::helpers::oauth2client;
use super::super::results::{BearerResult, BearerError};

pub fn command(config_dir: &str, client_name: &str) -> BearerResult<()> {
    debug!("Display authorization header for client {} in directory {}",
           client_name,
           config_dir);

    let mut conf = Config::from_file(config_dir, client_name)?;

    let update = match conf.expired() {
        Some(true) => {
            debug!("Refreshing Token");
            match conf.refresh_token() {
                Some(rtoken) => {
                    let tokens = oauth2client::from_refresh_token(&conf.client(), rtoken)?;
                    Some(tokens)
                }
                None => {
                    return Err(BearerError::ValueError("Client must be initialized. (No Refresh \
                                                        Tokens)"
                        .to_string()))
                }
            }
        }
        Some(false) => {
            debug!("Access Token Is OK");
            None
        }
        None => {
            error!("TODO: Command to reregister client implement yet");
            return Err(BearerError::ValueError("Client must be initialized".to_string()));
        }
    };

    if let Some(new_tokens) = update {
        conf.set_tokens(new_tokens);
        conf.write()?;
    }

    print!("Authorization: Bearer {}", conf.access_token().unwrap());
    io::stdout().flush().unwrap();

    Ok(())
}
