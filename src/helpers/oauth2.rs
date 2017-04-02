use std::time::Duration;
use std::io::Write;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};


use super::super::config::{Tokens, Config, ClientRef};
use super::super::results::{BearerResult, BearerError};
use super::oauth2client;

fn url_encode(to_encode: &str) -> String {
    to_encode.as_bytes().iter().fold(String::new(), |mut out, &b| {
        match b as char {
            // unreserved:
            'A'...'Z' | 'a'...'z' | '0'...'9' | '-' | '.' | '_' | '~' => out.push(b as char),

            ' ' => out.push('+'),

            ch => out.push_str(format!("%{:02X}", ch as usize).as_str()),
        };

        out
    })
}


struct Http<'a> {
    port: usize,
    client: ClientRef<'a>,
    tokens: Option<BearerResult<Tokens>>,
}

impl<'a> Http<'a> {
    pub fn new(config: &'a Config, port: usize) -> Self {
        Http {
            port: port,
            client: config.client(),
            tokens: None,
        }
    }

    pub fn fetch_tokens(&mut self) -> BearerResult<Tokens> {
        let listener = TcpListener::bind(self.addr().as_str()).unwrap();


        while self.tokens.is_none() {
            let stream = listener.incoming().next().unwrap();
            let mut stream = stream.unwrap();
            self.handle_client(&mut stream);
        }
        let tokens = self.tokens.as_ref().unwrap();
        match tokens.as_ref() {
            Ok(tokens) => Ok(tokens.clone()),
            Err(err) => Err(err.clone()),
        }
    }

    fn handle_client(&mut self, stream: &mut TcpStream) {
        stream.set_read_timeout(Some(Duration::new(15, 0))).unwrap();
        let mut buffer = [0; 4096];
        stream.read(&mut buffer[..]).unwrap();
        let httpquery = String::from_utf8_lossy(&buffer);
        // debug!("{}", string);

        // We don't bother the header
        let httpquery = httpquery.lines().next().unwrap();
        // debug!("{}", httpquery);

        let mut httpquery = httpquery.split_whitespace();
        let verb = httpquery.next().unwrap();
        if verb != "GET" {
            self.handle_405(stream);
            return;
        }

        let path = httpquery.next().unwrap();
        let mut path = path.split("?");
        let pathinfo = path.next().unwrap();

        if pathinfo != "/callback" {
            self.handle_404(stream);
            return;
        }

        let querystring = path.next();
        if querystring.is_none() {
            self.handle_302(stream);
            return;
        }

        let querystring = querystring.unwrap().split("&");
        for param in querystring {
            let mut param = param.split("=");
            let key = param.next();
            let value = param.next().unwrap();
            match key {
                Some("error") => self.handle_200_error(stream, value),
                Some("code") => self.handle_200_code(stream, value),
                _ => {}
            }
        }
    }

    fn handle_404(&mut self, stream: &mut TcpStream) {
        stream.write(b"HTTP/1.1 404 Not Found
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: 10

Not Found
")
            .unwrap();
    }

    fn handle_405(&mut self, stream: &mut TcpStream) {
        stream.write(b"HTTP/1.1 405 Method Not Allowed
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: 19

Method Not Allowed
")
            .unwrap();
    }

    fn handle_302(&mut self, stream: &mut TcpStream) {


        let mut location = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}",
            self.client.authorize_url,
            url_encode(self.client.client_id),
            url_encode(self.redirect_uri().as_ref()));

        if let Some(scope) = self.client.scope {
            location.push_str("&scope=");
            location.push_str(scope);
        }

        debug!("Redirecting to {}", location);

        let resp = format!("HTTP/1.1 302 Moved Temporarily
Connection: close
Server: bearer-rs
Location: {}
", location);
        stream.write(resp.as_bytes()).unwrap();
    }

    fn handle_200_code(&mut self, stream: &mut TcpStream, code: &str) {
        debug!("OAuth2.0 Authorization Code received, fetching tokens");

        let tokens = oauth2client::from_authcode(&self.client, code, self.redirect_uri().as_str());
        let content = match tokens {
            Ok(res) => {
                self.tokens = Some(Ok(res));
                "Token received".to_string()
            }
            Err(err) => {
                format!("Error while fetching token: {:?}", err)
            }
        };

        let resp = format!("HTTP/1.1 200 Ok
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: {}

{}",
                           content.len(),
                           content);

        stream.write(resp.as_bytes()).unwrap();


    }

    fn handle_200_error(&mut self, stream: &mut TcpStream, error: &str) {
        let content = format!("No Tokens returns. OAuth2.0 Authorization Server Error: {}",
                              error);
        let resp = format!("HTTP/1.1 200 Ok
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: {}

{}",
                           content.len(),
                           content);

        stream.write(resp.as_bytes()).unwrap();
        self.tokens = Some(Err(BearerError::OAuth2Error(content)));

    }

    fn addr(&self) -> String {
        return format!("127.0.0.1:{}", self.port);
    }

    fn redirect_uri(&self) -> String {
        return format!("http://localhost:{}/callback", self.port);
    }
}


pub fn get_tokens<'a>(config: &'a Config, port: usize) -> BearerResult<Tokens> {

    let mut server: Http<'a> = Http::new(config, port);
    let token = server.fetch_tokens()?;
    Ok(token)
}



#[cfg(test)]
mod tests {
    use std::thread;
    use std::time;
    use std::net::TcpStream;
    use rand::{thread_rng, Rng};

    use super::*;
    use super::super::super::results::BearerError;

    #[test]
    fn test_url_encode() {
        assert_eq!(url_encode("The éêè !"), "The+%C3%A9%C3%AA%C3%A8+%21")
    }

    #[test]
    fn test_get_tokens_ok() {
        let mut rng = thread_rng();
        let authorization_server_port: usize = rng.gen_range(3000, 9000);
        let client_port: usize = rng.gen_range(3000, 9000);
        let client_addr = format!("127.0.0.1:{}", client_port);
        let httphandler = thread::spawn(move || {
            let authorize = format!("http://localhost:{}/authorize", authorization_server_port);
            let token = format!("http://localhost:{}/token", authorization_server_port);
            let conf = Config::new(
                "/tmp",
                "client_name",
                "provider",
                authorize.as_str(),
                token.as_str(),
                "12e26",
                "secret",
                None).unwrap();

            let tokens = get_tokens(&conf, client_port);
            assert_eq!(tokens.is_ok(), true);
            let tokens = tokens.unwrap();
            assert_eq!(tokens.access_token, "atok");
            assert_eq!(tokens.refresh_token.unwrap(), "rtok");
            //assert_eq!(tokens.expires_at, "TIME DEPENDANT VALUE");
        });

        let dur = time::Duration::from_millis(700);
        thread::sleep(dur);

        let mut client = TcpStream::connect(client_addr.as_str()).unwrap();
        client.write_all(b"GET /callback HTTP/1.1\r\n\r\n").unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();
        assert_eq!(response, format!(r#"HTTP/1.1 302 Moved Temporarily
Connection: close
Server: bearer-rs
Location: http://localhost:{}/authorize?response_type=code&client_id=12e26&redirect_uri=http%3A%2F%2Flocalhost%3A{}%2Fcallback
"#, authorization_server_port, client_port));

        let authservhandler = thread::spawn(move || {
            let authorization_server = TcpListener::bind(format!("127.0.0.1:{}", authorization_server_port)).unwrap();
            let stream = authorization_server.incoming().next().unwrap();
            let mut stream = stream.unwrap();
            let tokens = r#"{
"access_token": "atok",
"expires_in": 42,
"refresh_token": "rtok"}"#;
            let resp = format!(r#"HTTP/1.0 200 Ok
Content-Type: application/json;
Content-Length: {}

{}"#,
                               tokens.len(),
                               tokens);
            stream.write(resp.as_bytes()).unwrap();
        });

        let dur = time::Duration::from_millis(700);
        thread::sleep(dur);

        let mut client = TcpStream::connect(client_addr).unwrap();
        client.write_all(b"GET /callback?code=abc HTTP/1.1\r\n\r\n").unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();

        assert_eq!(response, r#"HTTP/1.1 200 Ok
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: 14

Token received"#);

        // ensure threads are terminated
        httphandler.join().unwrap();
        authservhandler.join().unwrap();

    }

    #[test]
    fn test_get_tokens_error() {

        let mut rng = thread_rng();
        let client_port: usize = rng.gen_range(3000, 9000);
        let client_addr = format!("127.0.0.1:{}", client_port);

        let httphandler = thread::spawn(move || {
            let conf = Config::from_file("src/tests/conf", "dummy").unwrap();
            let tokens = get_tokens(&conf, client_port);
            assert_eq!(tokens.is_err(), true);
            let err = tokens.unwrap_err();
            assert_eq!(err, BearerError::OAuth2Error("".to_string()));
        });

        let dur = time::Duration::from_millis(700);
        thread::sleep(dur);

        let mut client = TcpStream::connect(client_addr.as_str()).unwrap();
        client.write_all(b"GET /callback?error=server_error&error_description=internal+server+error HTTP/1.1\r\n\r\n").unwrap();
        let mut response = String::new();
        client.read_to_string(&mut response).unwrap();

        assert_eq!(response, r#"HTTP/1.1 200 Ok
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: 68

No Tokens returns. OAuth2.0 Authorization Server Error: server_error"#);

        // ensure threads are terminated
        httphandler.join().unwrap();
    }

}