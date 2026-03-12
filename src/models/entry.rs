use std::cmp::Ordering;
use std::fmt::Display;
use std::ops::Sub;
use std::str::FromStr;

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};

use crate::{Status, TrackerError};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub struct Entry {
    #[deprecated(note = "id is not used anymore, will be removed in future versions")]
    #[serde(skip)]
    pub(crate) id: u8,

    pub(crate) status: Status,

    pub(crate) time: DateTime<Utc>,
}

impl Default for Entry {
    fn default() -> Self {
        #[allow(deprecated)]
        Self {
            id: 0,
            status: Status::Start,
            time: DateTime::default(),
        }
    }
}

impl Entry {
    #[deprecated(since = "3.0.0", note = "use direction methods instead of builder")]
    pub fn builder() -> EntryBuilder {
        EntryBuilder {
            inner: Default::default(),
            time_set: false,
        }
    }

    pub fn is_action(&self, status: Status) -> bool {
        self.status == status
    }

    pub fn new_now(status: Status) -> Self {
        Entry::builder()
            .status(status)
            .time(Utc::now())
            .build()
            .unwrap()
    }
}

impl FromStr for Entry {
    type Err = TrackerError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let entry: Entry = serde_json::from_str(s)?;
        Ok(entry)
    }
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

impl PartialOrd for Entry {
    fn partial_cmp(&self, other: &Entry) -> Option<Ordering> {
        self.time.partial_cmp(&other.time)
    }
}

impl Ord for Entry {
    fn cmp(&self, other: &Self) -> Ordering {
        self.time.cmp(&other.time)
    }
}

impl Sub for Entry {
    type Output = Duration;

    fn sub(self, rhs: Self) -> Self::Output {
        self.time - rhs.time
    }
}

#[derive(Clone)]
pub struct EntryBuilder {
    inner: Entry,
    time_set: bool,
}

impl EntryBuilder {
    /// Set id by incrementing the provided id
    #[deprecated(note = "id is not used anymore, will be removed in future versions")]
    pub fn id(&mut self, _id: u8) -> &mut Self {
        self
    }

    /// Set status of entry.
    pub fn status(&mut self, status: Status) -> &mut Self {
        self.inner.status = status;
        self
    }

    pub fn time(&mut self, time: DateTime<Utc>) -> &mut Self {
        self.inner.time = time;
        self.time_set = true;
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

            assert_eq!(Status::Start, entry.status);
        }

        #[test]
        fn should_build_entry_with_status() {
            let entry = Entry::builder()
                .time(DateTime::default())
                .status(Status::Break)
                .build()
                .unwrap();

            assert_eq!(Status::Break, entry.status);
        }
    }

    mod entry {
        use super::*;

        #[test]
        fn should_serialize() {
            let timestamp = chrono::DateTime::default();
            let expected_id = "\"id\":0";
            let expected_status = "\"status\":\"Start\"";
            let expected_time = format!("\"time\":\"{}", timestamp.format("%Y"));
            let entry_str = Entry::builder()
                .time(timestamp)
                .build()
                .unwrap()
                .to_string();

            assert!(!entry_str.contains(expected_id));
            assert!(entry_str.contains(expected_status));
            assert!(entry_str.contains(&expected_time));
        }

        #[test]
        fn should_deserialize() {
            let expected = chrono::Utc.with_ymd_and_hms(2022, 2, 4, 5, 27, 41).unwrap();
            let data = "{\"status\":\"Break\",\"time\":\"2022-02-04T05:27:41.000000000+00:00\"}";
            let entry = Entry::from_str(data).unwrap();

            assert_eq!(Status::Break, entry.status);
            assert_eq!(expected, entry.time);
        }

        #[test]
        #[allow(deprecated)]
        fn should_deserialize_deprecated() {
            let expected = chrono::Utc.with_ymd_and_hms(2022, 2, 4, 5, 27, 41).unwrap();
            let data =
                "{\"id\":2,\"status\":\"Disconnect\",\"time\":\"2022-02-04T05:27:41.000000000+00:00\"}";
            let entry = Entry::from_str(data).unwrap();

            assert_eq!(0, entry.id);
            assert_eq!(Status::Disconnect, entry.status);
            assert_eq!(expected, entry.time);
        }
    }
}
