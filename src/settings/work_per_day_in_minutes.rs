use chrono::{Date, DateTime, Local, Datelike};
use config::{Map, Value, ValueKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize, Serialize)]
#[allow(unused)]
pub struct WorkPerDayInMinutes {
    pub monday: u16,
    pub tuesday: u16,
    pub wednesday: u16,
    pub thursday: u16,
    pub friday: u16,
    pub saturday: u16,
    pub sunday: u16,
}

impl From<WorkPerDayInMinutes> for config::Value {
    fn from(w: WorkPerDayInMinutes) -> Self {
        let mut m = Map::new();
        m.insert(
            "monday".to_owned(),
            Value::new(Some(&"monday".to_owned()), ValueKind::U64(w.monday.into())),
        );
        m.insert(
            "tuesday".to_owned(),
            Value::new(
                Some(&"tuesday".to_owned()),
                ValueKind::U64(w.tuesday.into()),
            ),
        );
        m.insert(
            "wednesday".to_owned(),
            Value::new(
                Some(&"wednesday".to_owned()),
                ValueKind::U64(w.wednesday.into()),
            ),
        );
        m.insert(
            "thursday".to_owned(),
            Value::new(
                Some(&"thursday".to_owned()),
                ValueKind::U64(w.thursday.into()),
            ),
        );
        m.insert(
            "friday".to_owned(),
            Value::new(Some(&"friday".to_owned()), ValueKind::U64(w.friday.into())),
        );
        m.insert(
            "saturday".to_owned(),
            Value::new(
                Some(&"saturday".to_owned()),
                ValueKind::U64(w.saturday.into()),
            ),
        );
        m.insert(
            "sunday".to_owned(),
            Value::new(Some(&"sunday".to_owned()), ValueKind::U64(w.sunday.into())),
        );
        Value::new(Some(&"workperday".to_string()), ValueKind::Table(m))
    }
}

impl Default for WorkPerDayInMinutes {
    fn default() -> Self {
        Self {
            monday: 8 * 60,
            tuesday: 8 * 60,
            wednesday: 8 * 60,
            thursday: 8 * 60,
            friday: 8 * 60,
            saturday: 0,
            sunday: 0,
        }
    }
}

impl WorkPerDayInMinutes {
    pub fn from(&self, date: DateTime<Local>) -> &u16 {
        match date.weekday() {
            chrono::Weekday::Mon => &self.monday,
            chrono::Weekday::Tue => &self.tuesday,
            chrono::Weekday::Wed => &self.wednesday,
            chrono::Weekday::Thu => &self.thursday,
            chrono::Weekday::Fri => &self.friday,
            chrono::Weekday::Sat => &self.saturday,
            chrono::Weekday::Sun => &self.sunday,
        }
    }

    pub fn from_date(&self, date: Date<Local>) -> &u16 {
        match date.weekday() {
            chrono::Weekday::Mon => &self.monday,
            chrono::Weekday::Tue => &self.tuesday,
            chrono::Weekday::Wed => &self.wednesday,
            chrono::Weekday::Thu => &self.thursday,
            chrono::Weekday::Fri => &self.friday,
            chrono::Weekday::Sat => &self.saturday,
            chrono::Weekday::Sun => &self.sunday,
        }
    }
}
