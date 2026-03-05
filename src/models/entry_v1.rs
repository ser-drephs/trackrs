use std::str::FromStr;

use chrono::{ DateTime,  Utc };
use serde::{ Deserialize, Serialize };

use crate::models::{ Action, Entry };

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[allow(deprecated)]
#[deprecated = "use crate::models::Action instead"]
pub enum Status {
    Connect,
    Disconnect,
    Break,
    End,
    Takeover,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[deprecated = "use crate::models::Entry instead"]
#[allow(deprecated)]
pub struct EntryV1 {
    pub(crate) id: u8,

    #[allow(deprecated)]
    pub(crate) status: Status,

    pub(crate) time: DateTime<Utc>,
}

#[allow(deprecated)]
impl EntryV1 {
    pub fn upgrade(self) -> Entry {
        let action = match self.status {
            Status::Connect => Action::Start,
            Status::Disconnect => Action::Break,
            Status::Break => Action::Break,
            Status::End => Action::End,
            Status::Takeover => Action::Takeover,
        };
        Entry::new(action, self.time.to_utc())
    }
}

#[allow(deprecated)]
impl FromStr for EntryV1 {
    type Err = crate::TrackerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entry: EntryV1 = serde_json::from_str(s)?;
        Ok(entry)
    }
}
