use std::fs;
use std::path::Path;
use std::vec::Vec;
use tilde_expand::tilde_expand;

use super::super::results::{BearerResult, BearerError};


fn expand_path(config_dir: &str) -> BearerResult<String> {
    let config_dir_expanded = tilde_expand(config_dir.as_bytes());
    let config_dir_expanded = String::from_utf8(config_dir_expanded);
    if let Err(err) = config_dir_expanded {
        return Err(BearerError::UTF8EncodingError(format!(r#"Cannot build path from config dir \
{}: {:?}"#,
                                                          config_dir,
                                                          err)));
    }
    Ok(config_dir_expanded.unwrap())
}


pub fn build_path(config_dir: &str, client_name: &str) -> BearerResult<(String, bool)> {

    let config_dir_expanded = expand_path(config_dir)?;
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
            Err(BearerError::UTF8EncodingError(format!(r#"Could not build path with config dir {} \
and client {}"#,
                                                       config_dir_expanded,
                                                       client_name)))
        }
    }
}


pub fn list_clients(config_dir: &str) -> BearerResult<Vec<String>> {
    let config_dir_expanded = expand_path(config_dir)?;
    let path = config_dir_expanded.clone();
    let path = Path::new(path.as_str());
    if !path.is_dir() {
        return Err(BearerError::ValueError(format!("Path {} is not a directory",
                                                   config_dir_expanded)));
    }
    let paths = fs::read_dir(path).unwrap();
    let mut ret =
        paths.map(|pth| pth.unwrap().path().file_name().unwrap().to_str().unwrap().to_string())
            .filter(|pth| pth.ends_with(".toml"))
            .map(|pth| pth.as_str()[..pth.len() - 5].to_string())
            .collect::<Vec<String>>();
    ret.sort();
    Ok(ret)
}


#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;
    use std::path::Path;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_build_path_exists() {
        let (path, exists) = build_path("src/tests/conf", "dummy_with_tokens").unwrap();
        assert_eq!(exists, true);
        assert!(path.ends_with("src/tests/conf/dummy_with_tokens.toml"));
    }

    #[test]
    fn test_build_path_not_exists() {
        let (path, exists) = build_path("src/tests/conf", "not_exists").unwrap();
        assert_eq!(exists, false);
        assert!(path.ends_with("src/tests/conf/not_exists.toml"));
    }

    #[test]
    fn test_build_path_create_dir() {
        let rnd: String = thread_rng().gen_ascii_chars().take(10).collect();

        let tmpdir = format!("/tmp/test-bearer-{}", rnd);

        let dirpath = Path::new(tmpdir.as_str());
        assert_eq!(dirpath.exists(), false);

        let (path, exists) = build_path(tmpdir.as_str(), "not_exists").unwrap();
        assert_eq!(exists, false);

        let tmpdir = format!("/tmp/test-bearer-{}", rnd);
        let filepath = format!("{}/not_exists.toml", tmpdir);

        assert_eq!(path, filepath);

        let filepath = Path::new(filepath.as_str());

        assert_eq!(dirpath.exists(), true);
        assert_eq!(filepath.exists(), false);

        fs::remove_dir_all(tmpdir).unwrap();
    }

    #[test]
    fn test_list_clients_ok() {
        let clients = list_clients("src/tests/conf").unwrap();
        assert_eq!(clients.as_slice(),
                   &["dummy", "dummy_with_tokens", "invalid"])
    }

    #[test]
    fn test_list_clients_err() {
        let err = list_clients("not/an/existings/directory");
        assert_eq!(err.is_err(), true);
        let err = err.unwrap_err();
        assert_eq!(err, BearerError::ValueError("".to_string()))
    }
}
