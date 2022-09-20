use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeStatusError {
    #[error("time data does not contains any entries")]
    TimeDataEmpty,
    #[error("connect entry not found in time data")]
    ConnectNotFound,
    #[error("{r#type} is not provided for time status builder")]
    BuilderDataMissing { r#type: String },
    #[error("status error: {message}")]
    StatusError { message: String },
    #[error("try from integer error")]
    ConversionError(#[from] std::num::TryFromIntError),
    // #[error("io error")]
    // IoError(#[from] std::io::Error),
}
