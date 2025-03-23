use thiserror::Error;

#[derive(Error, Debug)]
pub enum UpgradeError {
    #[error("error upgrading to v1")]
    UpgradeV1Error(#[from] serde_json::Error),
}