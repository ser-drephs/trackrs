use chrono::NaiveDate;

use crate::{dto::TrackerData};

use super::StorageError;

pub trait Storage {
    /// Read single e
    fn read(&self, date: &NaiveDate) -> Result<TrackerData, StorageError>;
    fn read_all(&self) -> Result<Vec<TrackerData>, StorageError>;
    fn write(&self, data: &TrackerData) -> Result<(), StorageError>;
}
