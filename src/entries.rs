use serde::{ Deserialize, Serialize };

use crate::models::Entry;

const CURRENT_VERSION: u8 = 2;

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
