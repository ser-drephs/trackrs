use chrono::{ DateTime, Utc };
use serde::{ Deserialize, Serialize };

use crate::Status;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub(crate) status: Status,

    pub(crate) time: DateTime<Utc>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) note: Option<String>,
}

impl Entry {
    pub fn new(status: Status, time: DateTime<Utc>, note: Option<String>) -> Self {
        Self { status, time, note }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::Entry;
    use crate::Status;

    mod entry {
        use super::*;

        #[test]
        fn should_serialize() {
            let timestamp = chrono::DateTime::default();
            let expected_status = "\"status\":\"Connect\"";
            let expected_time = format!("\"time\":\"{}", timestamp.format("%Y"));
            let entry_str = serde_json
                ::to_string(&Entry::new(Status::Connect, timestamp, None))
                .unwrap();

            assert!(entry_str.contains(expected_status));
            assert!(entry_str.contains(&expected_time));
        }

        #[test]
        fn should_deserialize() {
            let expected = chrono::Utc.with_ymd_and_hms(2022, 2, 4, 5, 27, 41).unwrap();
            let data =
                "{\"status\":\"Disconnect\",\"time\":\"2022-02-04T05:27:41.000000000+00:00\"}";
            let entry: Entry = serde_json::from_str(data).unwrap();

            assert_eq!(Status::Disconnect, entry.status);
            assert_eq!(expected, entry.time);
            assert_eq!(None, entry.note);
        }

        #[test]
        fn should_roundtrip_note() {
            let entry = Entry::new(
                Status::Connect,
                chrono::DateTime::default(),
                Some("lunch".to_owned())
            );
            let entry: Entry = serde_json
                ::from_str(&serde_json::to_string(&entry).unwrap())
                .unwrap();

            assert_eq!(Some("lunch".to_owned()), entry.note);
        }

        #[test]
        fn should_not_serialize_empty_note() {
            let entry = Entry::new(Status::Connect, chrono::DateTime::default(), None);
            let entry_str = serde_json::to_string(&entry).unwrap();

            assert!(!entry_str.contains("note"));
        }
    }
}
