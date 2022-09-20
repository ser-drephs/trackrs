use super::TimeDataDaily;
use crate::time_data::WeeklyBuilder;

#[derive(Default, Clone, Debug)]
pub struct TimeDataWeekly {
    pub entries: Vec<TimeDataDaily>,
    pub week: u8,
    pub year: u16,
}

impl TimeDataWeekly {
    pub fn builder<'a>() -> WeeklyBuilder<'a> {
        WeeklyBuilder::default()
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
