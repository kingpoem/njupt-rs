use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("login failed: {0}")]
    Login(String),

    #[error("unexpected response: {0}")]
    Unexpected(String),

    #[error("missing field: {0}")]
    MissingField(&'static str),

    #[error("crypto error: {0}")]
    Crypto(String),

    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
}

pub type Result<T> = std::result::Result<T, Error>;
