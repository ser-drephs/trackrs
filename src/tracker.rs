use crate::{ config::ConfigurationProvider, models::{ Action, Entry, TrackerError }, storage::StorageProvider };

type TrackerResult = Result<(), TrackerError>;

pub struct Tracker {}

impl Tracker {
    pub fn add<P: StorageProvider>(provider: &P, action: Action) -> TrackerResult {
        let entry = Entry::new_now(action);
        let mut entries = provider.read()?;
        entries.add(entry);
        Ok(provider.write(&entries)?)
    }

    pub fn status<P: StorageProvider, C: ConfigurationProvider>(storage: &P, config: &C) -> TrackerResult {
        let mut entries = storage.read()?;
        entries.sort();

        Ok(())
    }

    pub fn status_week<P: StorageProvider, C: ConfigurationProvider>(provider: &P, config: &C, week: &u8, as_table: bool) -> TrackerResult {
        Ok(())
    }

    pub fn takeover<P: StorageProvider>(provider: &P) -> TrackerResult {
        Ok(())
    }
}
