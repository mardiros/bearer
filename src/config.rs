use std::fs::File;
use std::os::unix::fs::OpenOptionsExt;
use std::fs::OpenOptions;
use std::io::prelude::*;

use toml;
use toml::value::Datetime;
use chrono::Duration;
use chrono::datetime::DateTime;
use chrono::offset::utc::UTC;
use super::helpers::path::build_path;
use super::results::{BearerResult, BearerError};

#[derive(Debug, Serialize, Deserialize)]
struct TomlConfig {
    pub client: Client,
    pub tokens: Option<Tokens>,
}


#[derive(Debug, Serialize, Clone, Deserialize)]
struct Client {
    pub provider: String,
    pub token_url: String,
    pub authorize_url: String,
    pub client_id: String,
    pub secret: String,
}


#[derive(Debug, Serialize, Clone, Deserialize)]
pub struct Tokens {
    pub access_token: String,
    pub expires_at: Datetime,
    pub refresh_token: Option<String>,
}

impl Tokens {
    pub fn new(access_token: &str, expires_in: usize, refresh_token: Option<&str>) -> Self {
        let now: DateTime<UTC> = UTC::now();
        let duration = Duration::seconds(expires_in as i64);
        let expires_at = now + duration;
        let expires_at = expires_at.to_rfc3339().as_str().parse::<Datetime>().unwrap();
        Tokens {
            access_token: access_token.to_string(),
            expires_at: expires_at,
            refresh_token: match refresh_token {
                Some(token) => Some(token.to_string()),
                None => None,
            },
        }
    }
}


#[derive(Debug)]
pub struct Config {
    config_dir: String,
    client_name: String,
    path: String,
    config: TomlConfig,
}


impl Config {
    pub fn from_file(config_dir: &str, client_name: &str) -> BearerResult<Self> {

        let (path, exists) = build_path(config_dir, client_name)?;
        if !exists {
            return Err(BearerError::ValueError(format!("Client {} not registered", client_name)));
        }

        let file = File::open(path.as_str());
        if file.is_err() {
            return Err(BearerError::IOError(format!("Cannot open file {:?}: {:?}",
                                                    path,
                                                    file.err().unwrap())));
        }
        let mut file = file.unwrap();

        let mut buf: Vec<u8> = Vec::new();
        if let Err(err) = file.read_to_end(&mut buf) {
            return Err(BearerError::IOError(format!("Cannot read file {:?}: {:?}", path, err)));
        }

        let conf: Result<TomlConfig, toml::de::Error> = toml::from_slice(buf.as_slice());
        match conf {
            Ok(cf) => {
                Ok(Config {
                    config_dir: config_dir.to_string(),
                    client_name: client_name.to_string(),
                    path: path.to_owned(),
                    config: cf,
                })
            }
            Err(err) => {
                Err(BearerError::ParseError(format!("Cannot parse config file {}: {:?}",
                                                    path,
                                                    err)))
            }
        }
    }

    pub fn new(config_dir: &str,
               client_name: &str,
               provider: &str,
               authorize_url: &str,
               token_url: &str,
               client_id: &str,
               secret: &str)
               -> BearerResult<Self> {

        let (path, exists) = build_path(config_dir, client_name)?;
        if exists {
            return Err(BearerError::ValueError(format!("Client {} already registered",
                                                       client_name)));
        }

        let config = TomlConfig {
            client: Client {
                provider: provider.to_string(),
                authorize_url: authorize_url.to_string(),
                token_url: token_url.to_string(),
                client_id: client_id.to_string(),
                secret: secret.to_string(),
            },
            tokens: None,
        };

        Ok(Config {
            config_dir: config_dir.to_string(),
            client_name: client_name.to_string(),
            path: path.to_owned(),
            config: config,
        })
    }

    pub fn write(&self) -> BearerResult<()> {
        debug!("Writing configuration: {:?}", &self.config);
        let filecontent = toml::to_string(&self.config);
        if let Err(err) = filecontent {
            return Err(BearerError::SerializationError(format!("Cannot serialize configuration \
                                                                file {:?}: {:?}",
                                                               &self.config,
                                                               err)));
        }
        let filecontent = filecontent.unwrap();
        let file = OpenOptions::new()
            .mode(0o644)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.path.as_str());

        if let Err(err) = file {
            return Err(BearerError::SerializationError(format!("Cannot open configuration file \
                                                                {:?}: {:?}",
                                                               &self.config,
                                                               err)));
        }
        let mut file = file.unwrap();
        let written = file.write_all(filecontent.as_bytes());
        if let Err(err) = written {
            return Err(BearerError::IOError(format!("IOError while writing file {}: {}",
                                                    self.path.as_str(),
                                                    err)));
        }
        Ok(())
    }

    pub fn authorize_url(&self) -> &str {
        self.config.client.authorize_url.as_str()
    }

    pub fn token_url(&self) -> &str {
        self.config.client.token_url.as_str()
    }

    pub fn client_id(&self) -> &str {
        self.config.client.client_id.as_str()
    }

    pub fn secret(&self) -> &str {
        self.config.client.secret.as_str()
    }

    pub fn set_tokens(&mut self, tokens: Tokens) {
        self.config.tokens = Some(tokens)
    }

    pub fn access_token(&self) -> Option<&str> {
        match self.config.tokens {
            Some(ref tokens) => Some(tokens.access_token.as_str()),
            None => None,
        }
    }

    pub fn expires_at(&self) -> Option<DateTime<UTC>> {
        match self.config.tokens {
            Some(ref tokens) => {
                let expire_string = tokens.expires_at.to_string();
                Some(expire_string.parse::<DateTime<UTC>>().unwrap())
            }
            None => None,
        }
    }

    pub fn expired(&self) -> Option<bool> {
        match self.expires_at() {
            Some(date) => {
                let now: DateTime<UTC> = UTC::now();
                debug!("{:?} > {:?}: {}", now, date, now > date);
                Some(now > date)
            }
            None => None,
        }
    }

    pub fn refresh_token(&self) -> Option<&str> {
        match self.config.tokens {
            Some(ref tokens) => {
                match tokens.refresh_token {
                    Some(ref token) => Some(token.as_str()),
                    None => None,
                }
            }
            None => None,
        }
    }
}
