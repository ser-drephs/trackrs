use super::TimeDataDaily;
use crate::time_data::TimeDataWeeklyBuilder;

#[derive(Default, Clone)]
pub struct TimeDataWeekly {
    pub entries: Vec<TimeDataDaily>,
    pub week: i8,
}

impl TimeDataWeekly {
    pub fn builder() -> TimeDataWeeklyBuilder {
        TimeDataWeeklyBuilder::default()
    }
}
