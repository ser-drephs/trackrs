use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageError {
    // convert errors
    #[error("io error")]
    IoError(#[from] std::io::Error),

    #[error("parse error")]
    ParseError(#[from] serde_json::Error),

    // custom error messages
    #[error("file not found: {file}")]
    FileNotFound { file: String },
}
