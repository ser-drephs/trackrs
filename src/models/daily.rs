use crate::{time_data::DailyBuilder, Entry};


#[derive(Default, Clone, Debug)]
#[allow(dead_code)]
pub struct Daily {
    // todo: refactor
    pub entries: Vec<Entry>,
    // date: Option<Date<Local>>,
    // takeover: Option<Takeover>,
}

impl Daily {
    pub fn builder<'a>() -> DailyBuilder<'a> {
        DailyBuilder::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
