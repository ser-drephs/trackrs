use crate::models::{Entries, TrackerError};

pub trait Provider{
    fn read(&self) -> Result<Entries, TrackerError>;
    fn write(&self, entries: &Entries) -> Result<(), TrackerError>;
}