use chrono::{ DateTime, Duration, Utc };
use serde::{ Deserialize, Serialize };

use super::{ action, Action };
use std::{ cmp::Ordering, fmt, ops::Add };

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub struct Entry {
    action: Action,
    time: DateTime<Utc>,
}

impl Entry {
    pub fn new(action: Action, time: DateTime<Utc>) -> Self {
        Entry { action, time: time }
    }

    pub fn new_now(action: Action) -> Self {
        Entry::new(action, Utc::now())
    }

    pub fn add(&mut self, duration: Duration) -> Self {
        self.time = self.time.add(duration);
        *self
    }

    pub fn add_minutes(&mut self, duration: u16) -> Self {
        let dur = Duration::minutes(duration.into());
        self.add(dur)
    }

    pub fn is_action(&self, action: Action) -> bool {
        self.action == action
    }

    pub fn timestamp(&self) -> DateTime<Utc> {
        self.time
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

// impl FromStr for Entry {
//     type Err = serde_json::Error;

//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         let entry: Entry = serde_json::from_str(s)?;
//         Ok(entry)
//     }
// }

impl fmt::Display for Entry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", serde_json::to_string(&self).unwrap())
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ DateTime, Duration, TimeZone, Utc };

    use crate::models::{ Action, Entry };

    #[test]
    fn should_create_entry_for_now() {
        let entry = Entry::new_now(Action::Start);
        assert_eq!(Action::Start, entry.action)
    }

    #[test]
    fn should_create_entry_custom_time() {
        let time = DateTime::from(Utc.with_ymd_and_hms(2025, 1, 1, 15, 4, 0).unwrap());
        let entry = Entry::new(Action::Break, time);
        assert_eq!(Action::Break, entry.action);
        assert_eq!(time, entry.time)
    }

    #[test]
    fn should_serialize() {
        let time = DateTime::from(Utc.with_ymd_and_hms(2022, 2, 2, 15, 4, 12).unwrap());
        let entry = Entry::new(Action::End, time);
        let entry_string = serde_json::to_string(&entry).unwrap();
        assert_eq!("{\"action\":\"End\",\"time\":\"2022-02-02T15:04:12Z\"}", entry_string)
    }

    #[test]
    fn should_format_display() {
        let time = DateTime::from(Utc.with_ymd_and_hms(2022, 2, 2, 15, 4, 12).unwrap());
        let entry = Entry::new(Action::End, time);
        assert_eq!("{\"action\":\"End\",\"time\":\"2022-02-02T15:04:12Z\"}", format!("{}", entry))
    }

    #[test]
    fn should_sort_by_time() {
        let mut entries: Vec<Entry> = vec![
            Entry::new(
                Action::Start,
                DateTime::from(Utc.with_ymd_and_hms(2025, 1, 1, 15, 10, 0).unwrap())
            ),
            Entry::new(
                Action::Break,
                DateTime::from(Utc.with_ymd_and_hms(2025, 1, 1, 15, 2, 0).unwrap())
            ),
            Entry::new(
                Action::Start,
                DateTime::from(Utc.with_ymd_and_hms(2025, 1, 1, 15, 0, 0).unwrap())
            ),
            Entry::new(
                Action::End,
                DateTime::from(Utc.with_ymd_and_hms(2025, 1, 1, 16, 0, 0).unwrap())
            )
        ];
        entries.sort();

        assert_eq!("15:00:00", format!("{:?}", entries[0].time.time()));
        assert_eq!("15:02:00", format!("{:?}", entries[1].time.time()));
        assert_eq!("15:10:00", format!("{:?}", entries[2].time.time()));
        assert_eq!("16:00:00", format!("{:?}", entries[3].time.time()));
    }

    #[test]
    fn should_add_minutes_to_entry() {
        let left = Entry::new(
            Action::Start,
            DateTime::from(Utc.with_ymd_and_hms(2025, 1, 1, 8, 0, 0).unwrap())
        );
        let right = left.clone().add_minutes(482);
        assert_eq!("16:02:00", format!("{:?}", right.time.time()))
    }

    #[test]
    fn should_add_duration_to_entry() {
        let left = Entry::new(
            Action::Start,
            DateTime::from(Utc.with_ymd_and_hms(2025, 1, 1, 8, 0, 0).unwrap())
        );
        let right = left.clone().add(Duration::minutes(42));
        assert_eq!("08:42:00", format!("{:?}", right.time.time()))
    }
}
