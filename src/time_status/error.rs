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
    StatusError { message: String }, // #[error("parse error")]
                                     // ParseError(#[from] serde_json::Error),
                                     // #[error("io error")]
                                     // IoError(#[from] std::io::Error),
}
