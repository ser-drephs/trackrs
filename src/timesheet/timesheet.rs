use crate::{models::{Entries, Entry, TrackerError}, providers::Provider};

pub struct Timesheet{}

impl Timesheet {
    pub fn add<P: Provider>(provider: &P, entry: Entry) -> Result<(), TrackerError>{
        // TODO: read from existing file
        let mut entries = Entries::new();
        entries.add(entry);
        provider.write(&entries)
    }
}
