use time::{macros::date, OffsetDateTime, Weekday};

use crate::{
    models::{ Action, Entry, TrackerError },
    storage::StorageProvider,
};

type TrackerResult = Result<(), TrackerError>;

pub struct Tracker {}

impl Tracker {
    pub fn add<P: StorageProvider>(provider: &P, action: Action) -> TrackerResult {
        let entry = Entry::new_now(action);
        let mut entries = provider.read()?;
        entries.add(entry);
        Ok(provider.write(&entries)?)
    }

    pub fn status<P: StorageProvider>(storage: &P) -> TrackerResult {
        let mut timesheet = storage.read()?;
        timesheet.sort();
        let config = crate::config::Configuration::new()?;
        let work_time_today = config.workperday.from(OffsetDateTime::now_utc().weekday());
        let start = timesheet.first(Action::Start).unwrap();
        let end = match timesheet.first(Action::End){
            Some(res) => res,
            None => &Entry::new_now(Action::End),
        };

        Ok(())
    }

    pub fn status_week<P: StorageProvider>(
        provider: &P,
        week: &u8,
        as_table: bool
    ) -> TrackerResult {
        Ok(())
    }

    pub fn takeover<P: StorageProvider>(provider: &P) -> TrackerResult {
        Ok(())
    }
}
