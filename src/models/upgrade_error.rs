use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpgradeError {
    #[error("error reading entries")]
    ReadError(#[from] serde_json::Error),
    #[error("unsupported entries version {0}")]
    UnsupportedVersion(u8),
}
