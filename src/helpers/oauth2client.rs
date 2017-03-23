use std::io::prelude::*;

use serde_json;
use curl::easy::Easy;
use url::form_urlencoded::Serializer as URLSerializer;

use super::super::results::{BearerResult, BearerError};
use super::super::config::{Tokens, ClientRef};


#[derive(Deserialize)]
pub struct JsonToken {
    pub access_token: String,
    pub expires_in: Option<usize>,
    pub refresh_token: Option<String>,
}

pub fn fetch_token(token_url: &str, form: &mut &[u8]) -> BearerResult<Tokens> {

    let mut curl = Easy::new();
    curl.url(token_url).unwrap();
    curl.post(true).unwrap();
    curl.post_field_size(form.len() as u64).unwrap();

    let mut data = Vec::new();
    {
        let mut transfer = curl.transfer();

        transfer.read_function(|buf| Ok(form.read(buf).unwrap_or(0)))
            .unwrap();

        transfer.write_function(|new_data| {
                data.extend_from_slice(new_data);
                Ok(new_data.len())
            })
            .unwrap();

        transfer.perform().unwrap();
    }

    let code = curl.response_code().unwrap();
    let data = String::from_utf8(data).unwrap();

    if code >= 300 {
        return Err(BearerError::OAuth2Error(
            format!("Server did not return a valid response while consuming \
auth code, expected `2XX`, found `{}`: {}", code, data)));
    }


    let token: JsonToken = serde_json::from_str(data.as_str()).unwrap();

    Ok(Tokens::new(token.access_token.as_str(),
                   token.expires_in.unwrap_or(900),
                   match token.refresh_token {
                       Some(ref tok) => Some(tok.as_str()),
                       None => None,
                   }))

}

pub fn from_authcode(client: &ClientRef, authcode: &str) -> BearerResult<Tokens> {

    let form = URLSerializer::new(String::new())
        .append_pair("client_id", client.client_id)
        .append_pair("client_secret", client.secret)
        .append_pair("code", authcode)
        .append_pair("grant_type", "authorization_code")
        .finish();

    let mut form: &[u8] = form.as_bytes();
    fetch_token(client.token_url, &mut form)
}


pub fn from_refresh_token(client: &ClientRef, refresh_token: &str) -> BearerResult<Tokens> {

    let form = URLSerializer::new(String::new())
        .append_pair("client_id", client.client_id)
        .append_pair("client_secret", client.secret)
        .append_pair("refresh_token", refresh_token)
        .append_pair("grant_type", "refresh_token")
        .finish();

    let mut form: &[u8] = form.as_bytes();
    fetch_token(client.token_url, &mut form)
}
