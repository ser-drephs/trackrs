use serde::{ Deserialize, Serialize };

use crate::models::Entry;
use crate::models::Timesheet;

const CURRENT_VERSION: u8 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Entries {
    pub data: Vec<Entry>,
    pub version: u8,
}

impl Default for Entries {
    fn default() -> Self {
        Self { data: Default::default(), version: CURRENT_VERSION }
    }
}

impl Entries {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn append(&mut self, entry: &mut Vec<Entry>) -> &mut Self {
        self.data.append(entry);
        self
    }
}

impl Into<Timesheet> for Entries {
    fn into(self) -> Timesheet {
        let mut timesheet = Timesheet::new();
        timesheet.append(&mut self.data.clone().into_iter().map(|f| f.into()).collect());
        timesheet
    }
}

impl From<Timesheet> for Entries {
    fn from(timesheet: Timesheet) -> Entries {
        let mut entries = Entries::new();
        entries.append(&mut timesheet.data().clone().into_iter().map(|f| f.into()).collect());
        entries
    }
}
