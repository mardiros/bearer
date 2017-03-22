
#[derive(Debug, Clone)]
pub enum BearerError {
    ExpiredToken,
    ValueError(String),
    OAuth2Error(String),
    RecvError(String),
    IOError(String),
    ParseError(String),
    SerializationError(String),
    UTF8EncodingError(String),
}

pub type BearerResult<T> = Result<T, BearerError>;
