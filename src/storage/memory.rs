use std::{ cell::RefCell, collections::HashMap };

use chrono::{ NaiveDate, NaiveTime, TimeZone, Utc };

use crate::{ Entry, Status, StorageProvider, Timesheet, TrackerError };

/// Keeps timesheets in memory, to test code that loads and saves them without files.
#[derive(Default)]
pub(crate) struct MemoryStorage {
    days: RefCell<HashMap<NaiveDate, Timesheet>>,
}

impl MemoryStorage {
    /// Add the day `date` with `entries` as (status, hour, minute) in UTC.
    pub(crate) fn day(self, date: NaiveDate, entries: &[(Status, u32, u32)]) -> Self {
        let data = entries
            .iter()
            .map(|(status, h, m)| {
                let time = date.and_time(NaiveTime::from_hms_opt(*h, *m, 0).unwrap());
                Entry::new(status.to_owned(), Utc.from_utc_datetime(&time).fixed_offset(), None)
            })
            .collect();
        self.days.borrow_mut().insert(date, Timesheet { data, ..Default::default() });
        self
    }
}

impl StorageProvider for MemoryStorage {
    fn load(&self, date: NaiveDate) -> Result<Timesheet, TrackerError> {
        Ok(self.days.borrow().get(&date).cloned().unwrap_or_default())
    }

    fn save(&self, date: NaiveDate, timesheet: &Timesheet) -> Result<(), TrackerError> {
        self.days.borrow_mut().insert(date, timesheet.to_owned());
        Ok(())
    }
}
