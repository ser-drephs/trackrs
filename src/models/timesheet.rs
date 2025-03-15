use super::{ Action, Entry };
use serde_derive::{ Deserialize, Serialize };
use time::{ Duration, OffsetDateTime };

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timesheet {
    data: Vec<Entry>,
    version: u8,
}

impl Timesheet {
    pub fn new() -> Timesheet {
        Timesheet {
            data: vec![],
            version: 1,
        }
    }

    pub fn add(&mut self, entry: Entry) -> &mut Self {
        self.data.push(entry);
        self
    }

    pub fn sort(&mut self) -> &mut Self {
        self.data.sort();
        self
    }

    pub fn first(&self, action: Action) -> Option<&Entry> {
        self.data.iter().find(|d| d.action == action)
    }

    pub fn position(&self, action: Action) -> Option<usize> {
        self.data.iter().position(|d| d.action == action)
    }

    pub fn start_time(&self) -> OffsetDateTime {
        let start = self.first(Action::Start).unwrap();
        log::debug!("first '{}' entry found at '{}'", Action::Start, start);
        start.time
    }

    pub fn end_time(&self) -> OffsetDateTime {
        let end = match self.first(Action::End) {
            Some(res) => res,
            None => {
                log::debug!("no end found using current datetime");
                &Entry::new_now(Action::End)
            }
        };
        log::debug!("first '{}' entry found at '{}'", Action::End, end);
        end.time
    }

    pub fn break_time(&self) -> Option<OffsetDateTime> {
        let break_time = self.first(Action::Break);
        if break_time.is_some() {
            log::debug!("first '{}' found at '{}'", Action::Break, break_time.unwrap());
            Some(break_time.unwrap().time)
        } else {
            log::debug!("no break time found yet");
            None
        }
    }

    pub fn work_time(&self) -> Duration {
        let start = self.start_time();
        let end = self.end_time();
        let break_time = self.break_duration();
        let work_time = end - start - break_time;
        log::debug!("work time: {}", work_time);
        work_time
    }

    pub fn break_duration(&self) -> Duration {
        // TODO: calculate all breaks in timesheet
        todo!()
    }

    pub fn remaining_time(&self, expected: Duration) -> Duration {
        let start = self.start_time();
        let expected_end = start.checked_add(expected).unwrap();
        let remaining = expected_end - start;
        log::debug!("remaining time: {}", remaining);
        remaining
    }
}

#[cfg(test)]
mod tests {
    use time::{ macros::datetime, Duration, OffsetDateTime };

    use crate::models::{ Action, Entry };

    use super::Timesheet;

    #[test]
    fn should_add_entries() {
        let mut timesheet = Timesheet::new();
        timesheet.add(Entry::new(Action::Start, datetime!(2025-01-01 15:10 UTC)));
        timesheet.add(Entry::new(Action::Break, datetime!(2025-01-01 15:30 UTC)));
        timesheet.add(Entry::new(Action::End, datetime!(2025-01-01 16:00 UTC)));

        assert_eq!(3, timesheet.data.len());
    }

    #[test]
    fn should_sort_by_time() {
        let mut timesheet = Timesheet {
            version: 1,
            data: vec![
                Entry::new(Action::Start, datetime!(2025-01-01 15:10 UTC)),
                Entry::new(Action::Break, datetime!(2025-01-01 15:02 UTC)),
                Entry::new(Action::Start, datetime!(2025-01-01 15:00 UTC)),
                Entry::new(Action::End, datetime!(2025-01-01 16:00 UTC))
            ],
        };

        timesheet.sort();

        assert_eq!((15, 0, 0), timesheet.data[0].time.to_hms());
        assert_eq!((15, 2, 0), timesheet.data[1].time.to_hms());
        assert_eq!((15, 10, 0), timesheet.data[2].time.to_hms());
        assert_eq!((16, 0, 0), timesheet.data[3].time.to_hms());
    }

    #[test]
    fn should_calculate_work_time_with_end() {
        let mut timesheet = Timesheet {
            version: 1,
            data: vec![
                Entry::new(Action::Start, datetime!(2025-01-01 15:10 UTC)),
                Entry::new(Action::Break, datetime!(2025-01-01 15:02 UTC)),
                Entry::new(Action::Start, datetime!(2025-01-01 13:50 UTC)),
                Entry::new(Action::End, datetime!(2025-01-01 16:00 UTC))
            ],
        };

        timesheet.sort();
        let work_time = timesheet.work_time();
        assert_eq!(Duration::minutes(130), work_time)
    }

    #[test]
    fn should_calculate_work_time_without_end() {
        let now = OffsetDateTime::now_utc().checked_sub(Duration::minutes(38)).unwrap();
        let mut timesheet = Timesheet {
            version: 1,
            data: vec![Entry::new(Action::Start, now)],
        };

        timesheet.sort();
        let work_time = timesheet.work_time();
        // might be a small difference because of a second - that's why it's a range assert
        let range = 38..39;
        assert!(range.contains(&work_time.whole_minutes()))
    }
}
