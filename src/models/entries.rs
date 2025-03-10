use super::Entry;
use serde_derive::{ Deserialize, Serialize };

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entries {
    data: Vec<Entry>,
    version: u8,
}

impl Entries {
    pub fn new() -> Entries {
        Entries {
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
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use crate::models::{ Action, Entry };

    use super::Entries;

    #[test]
    fn should_add_entries() {
        let mut entries = Entries::new();
        entries.add(Entry::new(Action::Start, datetime!(2025-01-01 15:10 UTC)));
        entries.add(Entry::new(Action::Break, datetime!(2025-01-01 15:30 UTC)));
        entries.add(Entry::new(Action::End, datetime!(2025-01-01 16:00 UTC)));

        assert_eq!(3, entries.data.len());
    }

    #[test]
    fn should_sort_by_time() {
        let mut entries = Entries {
            version: 1,
            data: vec![
                Entry::new(Action::Start, datetime!(2025-01-01 15:10 UTC)),
                Entry::new(Action::Break, datetime!(2025-01-01 15:02 UTC)),
                Entry::new(Action::Start, datetime!(2025-01-01 15:00 UTC)),
                Entry::new(Action::End, datetime!(2025-01-01 16:00 UTC))
            ],
        };

        entries.sort();

        assert_eq!((15, 0, 0), entries.data[0].time().to_hms());
        assert_eq!((15, 2, 0), entries.data[1].time().to_hms());
        assert_eq!((15, 10, 0), entries.data[2].time().to_hms());
        assert_eq!((16, 0, 0), entries.data[3].time().to_hms());
    }
}
