use crate::{
    models::{Action, Entry, TrackerError},
    providers::Provider,
};

pub struct Timesheet {}

impl Timesheet {
    pub fn add<P: Provider>(provider: &P, action: Action) -> Result<(), TrackerError> {
        let entry = Entry::new_now(action);
        let mut entries = provider.read()?;
        entries.add(entry);
        Ok(provider.write(&entries)?)
    }
}
