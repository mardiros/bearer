
use super::super::config::Config;
use super::super::helpers::oauth2;
use super::super::helpers::oauth2client;
use super::super::results::BearerResult;

pub fn command(config_dir: &str, client_name: &str) -> BearerResult<()> {

    debug!("Refresh existing client {} in directory {}",
           client_name,
           config_dir);

    let mut conf = Config::from_file(config_dir, client_name)?;

    let tokens = match conf.refresh_token() {
        Some(rtoken) => {
            let tokens = oauth2client::from_refresh_token(&conf.client(), rtoken)?;
            debug!("Token retrieved usgin refresh token: {:?}", tokens);
            tokens
        }
        None => {
            println!("");
            println!("Visit to finish the configuration: http://localhost:6750/callback");

            debug!("Start server to retrieve tokens");
            let tokens = oauth2::get_tokens(&conf, 6750)?;
            debug!("Token retrieved using auth code: {:?}", tokens);
            tokens
        }
    };

    conf.set_tokens(tokens);
    conf.write()?;
    println!("Tokens retrieved succesfully");
    Ok(())

}