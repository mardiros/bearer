use std::time::Duration;
use std::io::Write;
use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};


use super::super::config::{Tokens, Config, ClientRef};
use super::super::results::{BearerResult, BearerError};
use super::oauth2client;

fn url_encode(to_decode: &str) -> String {
    to_decode.as_bytes().iter().fold(String::new(), |mut out, &b| {
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
    pub fn new(config: &'a Config) -> Self {
        Http {
            port: 6750,
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
        let resp = format!("HTTP/1.1 302 Moved Temporarily
Connection: close
Server: bearer-rs
Location: {}?response_type=code&client_id={}&redirect_uri={}
",
                           self.client.authorize_url,
                           url_encode(self.client.client_id),
                           url_encode(self.redirect_uri().as_ref()));

        stream.write(resp.as_bytes()).unwrap();
    }

    fn handle_200_code(&mut self, stream: &mut TcpStream, code: &str) {
        debug!("OAuth2.0 Authorization Code received, fetching tokens");

        stream.write(b"HTTP/1.1 200 OK
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: 17

Tokens received!
")
            .unwrap();

        let tokens = oauth2client::from_authcode(&self.client, code);
        self.tokens = Some(tokens);

    }

    fn handle_200_error(&mut self, stream: &mut TcpStream, error: &str) {
        let content = format!("No Tokens returns. OAuth2.0 Authorization Server Error: {}",
                              error);
        let resp = format!("HTTP/1.1 200 Ok
Connection: close
Server: bearer-rs
Content-Type: text/plain;charset=UTF-8
Content-Length: {}

{}
",
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


pub fn get_tokens<'a>(config: &'a Config) -> BearerResult<Tokens> {

    let mut server: Http<'a> = Http::new(config);
    let token = server.fetch_tokens();

    if let Err(err) = token {
        return Err(BearerError::RecvError(format!("cannot fetch tokens: {:?}", err)));
    }
    Ok(token.unwrap())
}
