use chrono::NaiveTime;
use serde::{Deserialize, Serialize};

use crate::{model::Status, Entry};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Item {
    id: u8,
    status: Status,
    time: NaiveTime,
}

#[allow(dead_code)]
impl Item {
    /// Creates a new [`Item`].
    pub fn new(id: u8, status: Status, time: NaiveTime) -> Item {
        Item { id, status, time }
    }

    pub fn id(&self) -> u8 {
        self.id
    }

    pub fn status(&self) -> Status {
        self.status
    }

    pub fn time(&self) -> NaiveTime {
        self.time
    }
}

impl From<&Entry> for Item {
    fn from(value: &Entry) -> Self {
        {
            Item {
                id: *value.id(),
                status: *value.status(),
                time: value.time().time(),
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use chrono::{Local, TimeZone};

    #[test]
    fn should_convert_from_entity() {
        let entry = Entry::builder()
            .id(0) // builder increases id on it's own
            .status(Status::Connect)
            .time(Local.ymd(2022, 2, 2).and_hms(8, 1, 30))
            .build()
            .unwrap();
        let item = Item::from(&entry);
        assert_eq!(1, item.id);
        assert_eq!(Status::Connect, item.status);
        assert_eq!(NaiveTime::from_hms(8, 1, 30), item.time);
    }
}
