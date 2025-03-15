use time::OffsetDateTime;

use crate::{
    config::Configuration, models::{ Action, Entry, TrackerError }, status::Daily, storage::StorageProvider
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

    pub fn status<P: StorageProvider>(storage: &P, config: &Configuration) -> TrackerResult {
        let mut timesheet = storage.read()?;
        timesheet.sort();
        let expected = config.workperday.into_duration(OffsetDateTime::now_utc().weekday());
        let daily = Daily{
            start_time: &timesheet.start_time(),
            end_time: &timesheet.end_time(),
            remaining_duration: &timesheet.remaining_time(expected),
            break_duration: &timesheet.break_time(),
            work_duration: &timesheet.work_time(),
            end_expected: todo!(),
            break_expected: todo!(),
        };

        print!("{}", daily);
        Ok(())
    }

    pub fn status_week<P: StorageProvider>(
        provider: &P,
        config: &Configuration,
        week: &u8,
        as_table: bool
    ) -> TrackerResult {
        Ok(())
    }

    pub fn takeover<P: StorageProvider>(provider: &P) -> TrackerResult {
        Ok(())
    }
}
