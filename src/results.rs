use std::error::Error;
use std::fmt;

#[derive(Debug, Clone)]
pub enum BearerError {
    ValueError(String),
    OAuth2Error(String),
    IOError(String),
    ParseError(String),
    SerializationError(String),
    UTF8EncodingError(String),
}

pub type BearerResult<T> = Result<T, BearerError>;


impl fmt::Display for BearerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}


impl Error for BearerError {
    fn description(&self) -> &str {
        match self {
            &BearerError::ValueError(_) => "Value Error",
            &BearerError::OAuth2Error(_) => "OAuth2 Error",
            &BearerError::IOError(_) => "IOError",
            &BearerError::ParseError(_) => "ParseError",
            &BearerError::SerializationError(_) => "SerializationError",
            &BearerError::UTF8EncodingError(_) => "UTF8EncodingError",
        }
    }

    fn cause(&self) -> Option<&Error> {
        None  // TOFIX
    }
}


#[cfg(test)]
impl PartialEq for BearerError {
    fn eq(&self, other: &BearerError) -> bool {
        self.description() == other.description()
    }
}
