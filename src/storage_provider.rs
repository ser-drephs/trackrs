use thiserror::Error;
use crate::models::Timesheet;
use crate::UpgradeError;

pub trait StorageProvider {
    fn read(&self) -> Result<Timesheet, StorageProviderError>;
    fn write(&self, data: &Timesheet) -> Result<(), StorageProviderError>;
}

#[derive(Error, Debug)]
pub enum StorageProviderError {
    #[error("parse error")]
    ParseError(#[from] serde_json::Error),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("model upgrade error")]
    UpgradeError(#[from] UpgradeError)
    // #[error("date format error")]
    // InvalidFormatDescription(#[from] time::error::InvalidFormatDescription),
    // #[error("date format error")]
    // FormatError(#[from] time::error::Format),
}