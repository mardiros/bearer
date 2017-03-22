use std::fs;
use std::path::Path;
use tilde_expand::tilde_expand;

use super::super::results::{BearerResult, BearerError};


pub fn build_path(config_dir: &str, client_name: &str) -> BearerResult<(String, bool)> {
    let config_dir_expanded = tilde_expand(config_dir.as_bytes());
    let config_dir_expanded = String::from_utf8(config_dir_expanded);
    if let Err(err) = config_dir_expanded {
        return Err(BearerError::UTF8EncodingError(format!("Cannot build path from config dir \
                                                           {}: {:?}",
                                                          config_dir,
                                                          err)));
    }
    let config_dir_expanded = config_dir_expanded.unwrap();
    let path = config_dir_expanded.clone();
    let path = Path::new(path.as_str());
    if !path.exists() {
        debug!("Creating the config dir {}, path not exists", config_dir);
        let res = fs::create_dir_all(&config_dir_expanded);
        if let Err(err) = res {
            return Err(BearerError::IOError(format!("Could not create directory {}: {}",
                                                    config_dir_expanded,
                                                    err)));
        }
    } else if !path.is_dir() {
        return Err(BearerError::ValueError(format!("Path {} is not a directory",
                                                   config_dir_expanded)));
    }


    let path = path.join(format!("{}.toml", client_name));

    match path.to_str() {
        Some(string) => Ok((string.to_string(), path.is_file())),
        None => {
            Err(BearerError::UTF8EncodingError(format!("Could not build path with config dir {} \
                                                        and client {}",
                                                       config_dir_expanded,
                                                       client_name)))
        }
    }
}
