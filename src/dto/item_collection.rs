use serde::{Deserialize, Serialize};

use crate::deprecated::Entry;

use super::item::Item;

#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct ItemCollection(Vec<Item>);

impl ItemCollection {
    fn new() -> ItemCollection {
        ItemCollection(Vec::new())
    }

    fn add(&mut self, elem: Item) {
        self.0.push(elem);
    }

    pub(super) fn len(&self) -> usize {
        self.0.len()
    }

    #[allow(dead_code)]
    pub(super) fn first(&self) -> Option<&Item> {
        self.0.first()
    }

    #[allow(dead_code)]
    pub(super) fn last(&self) -> Option<&Item> {
        self.0.last()
    }
}

impl FromIterator<Item> for ItemCollection {
    fn from_iter<I: IntoIterator<Item=Item>>(iter: I) -> Self {
        let mut c = ItemCollection::new();
        for i in iter {
            c.add(i);
        }
        c
    }
}

impl From<&Vec<Entry>> for ItemCollection {
    fn from(value: &Vec<Entry>) -> Self {
        value.iter()
            .map(Item::from)
            .collect::<ItemCollection>()
    }
}


#[cfg(test)]
mod tests {

    use crate::model::Status;

    use super::*;
    use chrono::{Local, TimeZone, NaiveTime};

    #[test]
    fn should_add_elements_to_collection(){
        let mut c = ItemCollection::new();
        assert_eq!(0, c.len());
        c.add(Item::new(3, Status::Break,NaiveTime::from_hms(8, 1, 30)));
        assert_eq!(1, c.len());
        assert_eq!(3, c.0.first().unwrap().id());
    }

    #[test]
    fn should_convert_from_entities() {
        let entry = Entry::builder()
            .id(0) // builder increases id on it's own
            .status(Status::Connect)
            .time(Local.ymd(2022, 2, 2).and_hms(8, 1, 30))
            .build()
            .unwrap();
        let entry2 = Entry::builder()
            .id(1) // builder increases id on it's own
            .status(Status::Break)
            .time(Local.ymd(2022, 2, 2).and_hms(10, 1, 30))
            .build()
            .unwrap();
        let items = ItemCollection::from(&vec![entry, entry2]);
        assert_eq!(2, items.len());
        assert!(items.0.last().unwrap().status() == Status::Break);
    }
}
