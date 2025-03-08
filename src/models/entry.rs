use chrono::{DateTime, Local};
use serde_derive::{Serialize, Deserialize};
use std::str::FromStr;
use super::Action;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub(crate) action: Action,
    pub(crate) time: DateTime<Local>,
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            action: Action::Start,
            time: DateTime::default(),
        }
    }
}

impl FromStr for Entry {
    type Err = serde_json::Error;

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