use crate::{Entry};

use super::daily_builder::DailyBuilder;

#[derive(Default, Clone, Debug)]
#[allow(dead_code)]
pub struct Daily {
    // todo: refactor
    pub(super) entries: Vec<Entry>,
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
