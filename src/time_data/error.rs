use thiserror::Error;

#[derive(Error, Debug)]
pub enum TimeDataError {
    #[error("{r#type} is not provided for time data builder")]
    BuilderDataMissing { r#type: String },
    #[error("parse error")]
    ParseError(#[from] serde_json::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
}
