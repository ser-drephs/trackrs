use chrono::{NaiveDate};
use serde::{Deserialize, Serialize};

use crate::deprecated::Entry;

use super::item_collection::ItemCollection;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ItemMetaData {
    pub date: NaiveDate,
    pub items: ItemCollection,
}

impl ItemMetaData{
    pub fn len(&self) -> usize {
        self.items.len()
    }
}

impl From<&Vec<Entry>> for ItemMetaData{
    fn from(values: &Vec<Entry>) -> Self {
        let date = match values.first() {
            Some(first) => first.time().date_naive(),
            None => todo!(),
        };
        ItemMetaData { date, items: ItemCollection::from(values) }
    }
}

#[cfg(test)]
mod tests {

    use crate::model::Status;

    use super::*;
    use chrono::{Local, TimeZone};

    #[test]
    fn should_convert_from_entities() {
        let entry = Entry::builder()
            .id(0) // builder increases id on it's own
            .status(Status::Connect)
            .time(Local.ymd(2022, 2, 5).and_hms(8, 1, 30))
            .build()
            .unwrap();
        let entry2 = Entry::builder()
            .id(1) // builder increases id on it's own
            .status(Status::Break)
            .time(Local.ymd(2022, 2, 5).and_hms(10, 1, 30))
            .build()
            .unwrap();
        let data = ItemMetaData::from(&vec![entry, entry2]);
        assert_eq!(2, data.len());
        let expected_date= NaiveDate::from_ymd_opt(2022, 2, 5).unwrap();
        assert_eq!(expected_date, data.date);
        assert!(data.items.last().unwrap().status() == Status::Break);
    }
}
