use crate::{Status, TrackerError};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub id: u8,
    pub status: Status,
    pub time: DateTime<Local>,
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
    pub fn new(id: u8, status: Status, time: DateTime<Local>) -> Entry {
        Entry { id, status, time }
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

#[cfg(test)]
mod tests {
    use crate::{Entry, Status};
    use chrono::TimeZone;
    use std::str::FromStr;

    #[test]
    fn should_serialize() {
        let timestamp = chrono::DateTime::default();
        let expected_id = "\"id\":0";
        let expected_status = "\"status\":\"Connect\"";
        let expected_time = format!("\"time\":\"{}", timestamp.format("%Y"));
        let entry_str = Entry::new(0, Status::Connect, timestamp).to_string();

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
