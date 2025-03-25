use thiserror::Error;

#[derive(Error, Debug)]
pub enum StorageProviderError {
    #[error("parse error")]
    ParseError(#[from] serde_json::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    // #[error("date format error")]
    // InvalidFormatDescription(#[from] time::error::InvalidFormatDescription),
    // #[error("date format error")]
    // FormatError(#[from] time::error::Format),
}


#[derive(Error, Debug)]
pub enum UpgradeError {
    #[error("error upgrading to v1")]
    UpgradeV1Error(#[from] serde_json::Error),
}