use std::str::FromStr;

use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};

use crate::TrackerError;

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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Connect,
    Disconnect,
    Break,
    End,
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

#[derive(Clone)]
pub struct EntryBuilder {
    inner: Entry,
    time_set: bool,
}

impl EntryBuilder {
    /// Set id by incrementing the provided id
    pub fn id(&mut self, id: u8) -> &mut Self {
        self.inner.id = id + 1;
        self
    }

    /// Set status of entry.
    pub fn status(&mut self, status: Status) -> &mut Self {
        self.inner.status = status;
        self
    }

    /// Build the entry.
    pub fn build(&mut self) -> Result<Entry, TrackerError> {
        if !self.time_set {
            Err(TrackerError::EntryError {
                message: "time not set".to_owned(),
            })
        } else {
            log::trace!("Build entry: {:?}", self.inner);
            Ok(self.inner.clone())
        }
    }

    pub fn time(&mut self, time: DateTime<Local>) -> &mut Self {
        self.inner.time = time;
        self.time_set = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use chrono::TimeZone;

    use super::{Entry, Status};

    mod builder {

        use chrono::DateTime;

        use super::*;

        #[test]
        fn should_build() {
            let entry = Entry::builder().time(DateTime::default()).build().unwrap();

            assert_eq!(Status::Connect, entry.status);
        }

        #[test]
        fn should_build_entry_with_status() {
            let entry = Entry::builder()
                .time(DateTime::default())
                .status(Status::Disconnect)
                .build()
                .unwrap();

            assert_eq!(Status::Disconnect, entry.status);
        }
    }

    mod entry {

        use super::*;

        #[test]
        fn should_serialize() {
            let timestamp = chrono::DateTime::default();
            let expected_id = "\"id\":0";
            let expected_status = "\"status\":\"Connect\"";
            let expected_time = format!("\"time\":\"{}", timestamp.format("%Y"));
            let entry_str = Entry::builder()
                .time(timestamp)
                .build()
                .unwrap()
                .to_string();

            assert!(entry_str.contains(expected_id));
            assert!(entry_str.contains(expected_status));
            assert!(entry_str.contains(&expected_time));
        }

        #[test]
        fn should_deserialize() {
            let expected = chrono::Local
                .ymd(2022, 2, 4)
                .and_hms_nano(5, 27, 41, 200000000);
            let data =
                "{\"id\":2,\"status\":\"Disconnect\",\"time\":\"2022-02-04T05:27:41.200000000+00:00\"}";
            let entry = Entry::from_str(data).unwrap();

            assert_eq!(2, entry.id);
            assert_eq!(Status::Disconnect, entry.status);
            assert_eq!(expected, entry.time);
        }
    }
}
