use super::Entry;
use serde_derive::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Entries {
    data: Vec<Entry>,
    version: u8
}

impl Entries {
    pub fn new() -> Entries {
        Entries { data: vec![], version: 1 }
    }

    pub fn add(&mut self, entry: Entry) -> &mut Self {
        self.data.push(entry);
        self
    }

    pub fn sort_by(&mut self) -> &mut Self {
        self.data
            .sort_by(|a: &Entry, b: &Entry| a.partial_cmp(&b).unwrap());
        self
    }
}
