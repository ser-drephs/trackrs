use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeDataError {
    #[error("time data root is not provided for time data builder")]
    RootNotProvided,
    #[error("date is not provided for time data builder")]
    DateNotProvided,
    #[error("parse error")]
    ParseError(#[from] serde_json::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
}
