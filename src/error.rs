use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum Error {
    #[error("Usage: subdomain_scanner <google.com>")]
    CliUsage,

    #[error("Reqwest: {0}")]
    Reqwest(String),
}

impl std::convert::From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Error::Reqwest(err.to_string())
    }
}
