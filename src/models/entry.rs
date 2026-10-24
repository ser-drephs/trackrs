use chrono::{ DateTime, FixedOffset };
use serde::{ Deserialize, Serialize };

use crate::Status;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub(crate) status: Status,

    /// Local time of the entry with its offset from UTC, e.g. `2022-02-07T08:00:00+01:00`.
    pub(crate) time: DateTime<FixedOffset>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) note: Option<String>,
}

impl Entry {
    pub fn new(status: Status, time: DateTime<FixedOffset>, note: Option<String>) -> Self {
        Self { status, time, note }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ DateTime, FixedOffset, TimeZone };

    use super::Entry;
    use crate::Status;

    fn at(offset_hours: i32, hour: u32, minute: u32) -> DateTime<FixedOffset> {
        FixedOffset::east_opt(offset_hours * 3600)
            .unwrap()
            .with_ymd_and_hms(2022, 2, 4, hour, minute, 0)
            .unwrap()
    }

    mod entry {
        use super::*;

        #[test]
        fn should_serialize_with_its_offset() {
            let entry = Entry::new(Status::Connect, at(1, 8, 30), None);
            let entry_str = serde_json::to_string(&entry).unwrap();

            assert!(entry_str.contains("\"status\":\"Connect\""), "{entry_str}");
            assert!(entry_str.contains("\"time\":\"2022-02-04T08:30:00+01:00\""), "{entry_str}");
        }

        #[test]
        fn should_serialize_with_a_negative_offset() {
            let entry = Entry::new(Status::Connect, at(-5, 8, 30), None);
            let entry_str = serde_json::to_string(&entry).unwrap();

            assert!(entry_str.contains("\"time\":\"2022-02-04T08:30:00-05:00\""), "{entry_str}");
        }

        #[test]
        fn should_deserialize_utc_time() {
            for time in ["2022-02-04T05:27:41Z", "2022-02-04T05:27:41.000000000+00:00"] {
                let data = format!("{{\"status\":\"Disconnect\",\"time\":\"{time}\"}}");
                let entry: Entry = serde_json::from_str(&data).unwrap();

                assert_eq!(Status::Disconnect, entry.status);
                assert_eq!(FixedOffset::east_opt(0).unwrap().with_ymd_and_hms(2022, 2, 4, 5, 27, 41).unwrap(), entry.time);
                assert_eq!(0, entry.time.offset().local_minus_utc());
                assert_eq!(None, entry.note);
            }
        }

        #[test]
        fn should_deserialize_time_with_offset() {
            let data = "{\"status\":\"Connect\",\"time\":\"2022-02-04T08:30:00+01:00\"}";
            let entry: Entry = serde_json::from_str(data).unwrap();

            assert_eq!(at(1, 8, 30), entry.time);
            // The offset is kept, not converted, so the local time stays 08:30.
            assert_eq!(3600, entry.time.offset().local_minus_utc());
            assert_eq!("08:30", entry.time.format("%H:%M").to_string());
            // The same instant as 07:30 UTC.
            assert_eq!(at(0, 7, 30), entry.time);
        }

        #[test]
        fn should_roundtrip_note() {
            let entry = Entry::new(Status::Connect, at(1, 8, 30), Some("lunch".to_owned()));
            let entry: Entry = serde_json
                ::from_str(&serde_json::to_string(&entry).unwrap())
                .unwrap();

            assert_eq!(Some("lunch".to_owned()), entry.note);
        }

        #[test]
        fn should_roundtrip_offset() {
            let entry = Entry::new(Status::Break, at(-5, 14, 0), None);
            let entry: Entry = serde_json
                ::from_str(&serde_json::to_string(&entry).unwrap())
                .unwrap();

            assert_eq!(-5 * 3600, entry.time.offset().local_minus_utc());
            assert_eq!(at(-5, 14, 0), entry.time);
        }

        #[test]
        fn should_not_serialize_empty_note() {
            let entry = Entry::new(Status::Connect, at(1, 8, 30), None);
            let entry_str = serde_json::to_string(&entry).unwrap();

            assert!(!entry_str.contains("note"));
        }
    }
}
