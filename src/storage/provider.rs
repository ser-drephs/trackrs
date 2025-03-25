use crate::models::Timesheet;

use super::StorageProviderError;

pub trait StorageProvider {
    fn read(&self) -> Result<Timesheet, StorageProviderError>;
    fn write(&self, data: &Timesheet) -> Result<(), StorageProviderError>;
}
