use super::Entry;

pub struct Entries {
    data: Vec<Entry>
}

impl Entries{
    pub fn new() -> Entries {
        Entries{data: vec!()}
    }

    pub fn add(&mut self, entry: Entry) -> &mut Self{
        &self.data.push(entry);
        self
    }
}