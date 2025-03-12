use super::{ Action, Entry };
use serde_derive::{ Deserialize, Serialize };

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
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

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
}
