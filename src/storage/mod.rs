use chrono::NaiveDate;

use crate::{ Timesheet, TrackerError };

mod json;
pub use json::*;

/// Where timesheets are kept, one per day.
pub trait StorageProvider {
    /// The timesheet of `date`, an empty one if none was stored yet.
    fn load(&self, date: NaiveDate) -> Result<Timesheet, TrackerError>;

    /// Store `timesheet` as the one of `date`, replacing a stored one.
    fn save(&self, date: NaiveDate, timesheet: &Timesheet) -> Result<(), TrackerError>;
}
