use crate::models::{Entries, TrackerError};

use super::Provider;

pub struct JsonProvider{}

impl Provider for JsonProvider{
    fn read(&self) -> Result<Entries, TrackerError>{
        log::debug!("read using json provider");
        Ok(Entries::new())
    }

    fn write(&self, entries: &Entries) -> Result<(), TrackerError>{
        log::debug!("write using json provider");
        Ok(())
    }
}