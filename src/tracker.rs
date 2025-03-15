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
        let break_limit = config.limits.limit_by_start();

        let daily = Daily{
            start_time: &timesheet.start_time(),
            end_time: &timesheet.end_time(),
            break_time: timesheet.break_time().as_ref(),
            remaining_duration: &timesheet.remaining_time(expected),
            break_duration: &timesheet.break_duration(),
            work_duration: &timesheet.work_time(),
            end_expected: &timesheet.end_time().checked_add(expected).unwrap(),
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
