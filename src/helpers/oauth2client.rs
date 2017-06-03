
use serde_json;
use cabot::{RequestBuilder, Client};
use url::form_urlencoded::Serializer as URLSerializer;

use super::super::results::{BearerResult, BearerError};
use super::super::config::{Tokens, ClientRef};


#[derive(Deserialize)]
pub struct JsonToken {
    pub access_token: String,
    pub expires_in: Option<usize>,
    pub refresh_token: Option<String>,
}


fn fetch_token(token_url: &str, form: &[u8]) -> BearerResult<Tokens> {
    println!("Fetchin tokens from {}", token_url);
    let request = RequestBuilder::new(token_url)
        .set_http_method("POST")
        .add_header("Content-Type: application/x-www-form-urlencoded")
        .set_body(form)
        .build();

    let request = request.unwrap();

    let client = Client::new();
    let response = client.execute(&request).unwrap();

    let code = response.status_code();
    let data = response.body_as_string().unwrap();

    if code >= 300 {
        return Err(BearerError::OAuth2Error(format!(r#"Server did not return a valid response \
while consuming auth code, expected `2XX`, found `{}`: {}"#,
                                                    code,
                                                    data)));
    }


    let token: JsonToken = serde_json::from_str(data.as_str()).unwrap();

    Ok(Tokens::new(token.access_token.as_str(),
                   token.expires_in.unwrap_or(900),
                   match token.refresh_token {
                       Some(ref tok) => Some(tok.as_str()),
                       None => None,
                   }))

}


pub fn from_authcode(client: &ClientRef,
                     authcode: &str,
                     callback_uri: &str)
                     -> BearerResult<Tokens> {

    let form = URLSerializer::new(String::new())
        .append_pair("client_id", client.client_id)
        .append_pair("client_secret", client.secret)
        .append_pair("code", authcode)
        .append_pair("redirect_uri", callback_uri)
        .append_pair("grant_type", "authorization_code")
        .finish();

    fetch_token(client.token_url, form.as_bytes())
}


pub fn from_refresh_token(client: &ClientRef, refresh_token: &str) -> BearerResult<Tokens> {

    let form = URLSerializer::new(String::new())
        .append_pair("client_id", client.client_id)
        .append_pair("client_secret", client.secret)
        .append_pair("refresh_token", refresh_token)
        .append_pair("grant_type", "refresh_token")
        .finish();

    let mut token = fetch_token(client.token_url, form.as_bytes())?;
    if token.refresh_token.is_none() {
        token.refresh_token = Some(refresh_token.to_string());
    }
    Ok(token)
}

#[cfg(test)]
mod tests {
    use std::io::prelude::*;

    use std::thread;
    use std::time;
    use std::net::TcpListener;
    use rand::{thread_rng, Rng};

    use super::*;
    use super::super::super::config::ClientRef;

    #[test]
    fn test_from_authcode() {

        let mut rng = thread_rng();
        let server_port: usize = rng.gen_range(3000, 9000);
        let server_addr = format!("127.0.0.1:{}", server_port);
        let token_url = format!("http://127.0.0.1:{}", server_port);

        let authservhandler = thread::spawn(move || {
            let authorization_server = TcpListener::bind(server_addr.as_str()).unwrap();
            let stream = authorization_server.incoming().next().unwrap();
            let mut stream = stream.unwrap();
            let tokens = r#"{
"access_token": "atok",
"expires_in": 42,
"refresh_token": "rtok"}"#;
            let content_len = format!("Content-Length: {}", tokens.len());

            let resp = vec!["HTTP/1.0 200 Ok",
                            "Content-Type: application/json",
                            content_len.as_str(),
                            "",
                            tokens];
            let resp = resp.join("\r\n");

            stream.write(resp.as_bytes()).unwrap();
        });

        let dur = time::Duration::from_millis(700);
        thread::sleep(dur);

        let client = ClientRef {
            provider: "",
            token_url: token_url.as_str(),
            authorize_url: "",
            client_id: "",
            secret: "",
            scope: None,
        };

        let tokens = from_authcode(&client, "authcode", "http://127.0.0.1/callback");
        assert_eq!(tokens.is_err(), false);
        let tokens = tokens.unwrap();
        assert_eq!(tokens.access_token, "atok");
        // assert_eq!(tokens.expires_at, now() + 42...);
        assert_eq!(tokens.refresh_token, Some("rtok".to_string()));
        authservhandler.join().unwrap();

    }

    #[test]
    fn test_from_refresh_token() {

        let mut rng = thread_rng();
        let server_port: usize = rng.gen_range(3000, 9000);
        let server_addr = format!("127.0.0.1:{}", server_port);
        let token_url = format!("http://127.0.0.1:{}", server_port);

        let authservhandler = thread::spawn(move || {
            let authorization_server = TcpListener::bind(server_addr.as_str()).unwrap();
            let stream = authorization_server.incoming().next().unwrap();
            let mut stream = stream.unwrap();
            let tokens = r#"{
"access_token": "atok",
"expires_in": 42}"#;
            let content_len = format!("Content-Length: {}", tokens.len());
            let resp = vec!["HTTP/1.0 200 Ok",
                            "Content-Type: application/json",
                            content_len.as_str(),
                            "",
                            tokens];
            let resp = resp.join("\r\n");

            stream.write(resp.as_bytes()).unwrap();
        });

        let dur = time::Duration::from_millis(700);
        thread::sleep(dur);

        let client = ClientRef {
            provider: "",
            token_url: token_url.as_str(),
            authorize_url: "",
            client_id: "",
            secret: "",
            scope: None,
        };

        let tokens = from_refresh_token(&client, "refresh_token");
        assert_eq!(tokens.is_err(), false);
        let tokens = tokens.unwrap();
        assert_eq!(tokens.access_token, "atok");
        assert_eq!(tokens.refresh_token, Some("refresh_token".to_string()));
        authservhandler.join().unwrap();

    }


}
