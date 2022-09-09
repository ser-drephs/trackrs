use std::str::FromStr;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::{Status, EntryBuilder, TrackerError};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub(crate) id: u8,

    pub(crate) status: Status,

    pub(crate) time: DateTime<Local>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            id: 0,
            status: Status::Connect,
            time: DateTime::default(),
        }
    }
}

impl Entry {
    pub fn builder() -> EntryBuilder {
        EntryBuilder {
            inner: Default::default(),
            time_set: false,
        }
    }
}

impl FromStr for Entry {
    type Err = TrackerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entry: Entry = serde_json::from_str(s)?;
        Ok(entry)
    }
}

impl ToString for Entry {
    fn to_string(&self) -> String {
        serde_json::to_string(&self).unwrap()
    }
}
