use super::Action;
use serde_derive::{Deserialize, Serialize};
use std::{cmp::Ordering, str::FromStr};
use time::OffsetDateTime;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Entry {
    pub(crate) action: Action,
    #[serde(with = "time::serde::rfc3339")]
    pub(crate) time: OffsetDateTime,
}

impl Entry {
    pub fn new_now(action: Action) -> Self {
        Entry {
            action,
            time: OffsetDateTime::now_utc(),
        }
    }

    pub fn new(action: Action, time: OffsetDateTime) -> Self {
        Entry { action, time }
    }
}

impl Default for Entry {
    fn default() -> Self {
        Self {
            action: Action::Start,
            time: OffsetDateTime::now_utc(),
        }
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

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use crate::models::{Action, Entry};

    #[test]
    fn should_create_entry_for_now() {
        crate::test::setup();
        let entry = Entry::new_now(Action::Start);
        assert_eq!(Action::Start, entry.action)
    }

    #[test]
    fn should_create_entry_custom_time() {
        crate::test::setup();
        let time = datetime!(2025-01-01 15:04 UTC);
        let entry = Entry::new(Action::Break, time);
        assert_eq!(Action::Break, entry.action);
        assert_eq!(time, entry.time)
    }

    #[test]
    fn should_serialize() {
        crate::test::setup();
        let time = datetime!(2022-02-02 15:04:12 UTC);
        let entry = Entry::new(Action::End, time);
        let entry_string = serde_json::to_string(&entry).unwrap();
        assert_eq!(
            "{\"action\":\"End\",\"time\":\"2022-02-02T15:04:12Z\"}",
            entry_string
        )
    }

    #[test]
    fn should_sort_by_time() {
        crate::test::setup();
        let mut entries: Vec<Entry> = vec![
            Entry::new(Action::Start, datetime!(2025-01-01 15:10 UTC)),
            Entry::new(Action::Break, datetime!(2025-01-01 15:02 UTC)),
            Entry::new(Action::Start, datetime!(2025-01-01 15:00 UTC)),
            Entry::new(Action::End, datetime!(2025-01-01 16:00 UTC)),
        ];
        entries.sort();

        assert_eq!((15, 0, 0), entries[0].time.to_hms());
        assert_eq!((15, 2, 0), entries[1].time.to_hms());
        assert_eq!((15, 10, 0), entries[2].time.to_hms());
        assert_eq!((16, 0, 0), entries[3].time.to_hms());
    }
}
