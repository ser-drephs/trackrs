use serde::{Deserialize, Serialize};

use crate::Entry;

const CURRENT_VERSION: u8 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timesheet {
    pub data: Vec<Entry>,
    pub version: u8,
}

impl Timesheet {
    pub fn new() -> Self {
        Timesheet { data: vec!(), version: CURRENT_VERSION }
    }

    pub fn append(&mut self, entry: &mut Vec<Entry>) -> &mut Self {
        self.data.append(entry);
        self
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn sort(&mut self) -> &mut Self{
        self.data.sort();
        self
    }
}