use std::str::FromStr;

use chrono::{DateTime, Local, Utc};
use serde::{Deserialize, Serialize};

use crate::{Action, Entry, TrackerError};


#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[deprecated = "use crate::models::Action instead"]
pub enum Status {
    Connect,
    Disconnect,
    Break,
    End,
    Takeover
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
#[deprecated = "use crate::models::Entry instead"]
pub struct EntryV1 {
    pub(crate) id: u8,

    pub(crate) status: Status,

    pub(crate) time: DateTime<Local>,
}

impl EntryV1 {
    pub fn builder() -> EntryBuilder {
        EntryBuilder {
            inner: EntryV1 { status: Status::Connect, id: 1, time: Local::now()},
            time_set: false,
        }
    }

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

impl FromStr for EntryV1 {
    type Err = TrackerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entry: EntryV1 = serde_json::from_str(s)?;
        Ok(entry)
    }
}

#[derive(Clone)]
pub struct EntryBuilder {
    inner: EntryV1,
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
    pub fn build(&mut self) -> Result<EntryV1, TrackerError> {
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

    use super::{EntryV1, Status};

    mod builder {

        use chrono::DateTime;

        use super::*;

        #[test]
        fn should_build() {
            let entry = EntryV1::builder().time(DateTime::default()).build().unwrap();

            assert_eq!(Status::Connect, entry.status);
        }

        #[test]
        fn should_build_entry_with_status() {
            let entry = EntryV1::builder()
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
            let entry_str = EntryV1::builder()
                .time(timestamp)
                .build()
                .unwrap();
                // .to_string();

            // assert!(entry_str.contains(expected_id));
            // assert!(entry_str.contains(expected_status));
            // assert!(entry_str.contains(&expected_time));
        }

        #[test]
        fn should_deserialize() {
            let expected = chrono::Utc
                .with_ymd_and_hms(2022, 2, 4,5, 27, 41).unwrap();
            let data =
                "{\"id\":2,\"status\":\"Disconnect\",\"time\":\"2022-02-04T05:27:41.000000000+00:00\"}";
            let entry = EntryV1::from_str(data).unwrap();

            assert_eq!(2, entry.id);
            assert_eq!(Status::Disconnect, entry.status);
            assert_eq!(expected, entry.time);
        }
    }
}
