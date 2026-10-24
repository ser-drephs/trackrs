use chrono::Datelike;
use config::{ Map, Value, ValueKind };
use serde::Serialize;
use serde_derive::Deserialize;

/// Target work time of each weekday, in minutes.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq)]
#[allow(unused)]
pub struct WorkPerDay {
    pub monday: u16,
    pub tuesday: u16,
    pub wednesday: u16,
    pub thursday: u16,
    pub friday: u16,
    pub saturday: u16,
    pub sunday: u16,
}

impl From<WorkPerDay> for config::Value {
    fn from(w: WorkPerDay) -> Self {
        let mut m = Map::new();
        m.insert(
            "monday".to_owned(),
            Value::new(Some(&"monday".to_owned()), ValueKind::U64(w.monday.into()))
        );
        m.insert(
            "tuesday".to_owned(),
            Value::new(Some(&"tuesday".to_owned()), ValueKind::U64(w.tuesday.into()))
        );
        m.insert(
            "wednesday".to_owned(),
            Value::new(Some(&"wednesday".to_owned()), ValueKind::U64(w.wednesday.into()))
        );
        m.insert(
            "thursday".to_owned(),
            Value::new(Some(&"thursday".to_owned()), ValueKind::U64(w.thursday.into()))
        );
        m.insert(
            "friday".to_owned(),
            Value::new(Some(&"friday".to_owned()), ValueKind::U64(w.friday.into()))
        );
        m.insert(
            "saturday".to_owned(),
            Value::new(Some(&"saturday".to_owned()), ValueKind::U64(w.saturday.into()))
        );
        m.insert(
            "sunday".to_owned(),
            Value::new(Some(&"sunday".to_owned()), ValueKind::U64(w.sunday.into()))
        );
        Value::new(Some(&"workperday".to_string()), ValueKind::Table(m))
    }
}

impl Default for WorkPerDay {
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

impl WorkPerDay {
    /// Target work time in minutes of the weekday of `date`.
    pub fn minutes_on(&self, date: impl Datelike) -> u16 {
        [
            self.monday,
            self.tuesday,
            self.wednesday,
            self.thursday,
            self.friday,
            self.saturday,
            self.sunday,
        ][date.weekday().num_days_from_monday() as usize]
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ NaiveDate, TimeZone, Utc };
    use config::Value;

    use super::WorkPerDay;

    /// Each weekday with a different value, to tell them apart.
    fn varied() -> WorkPerDay {
        WorkPerDay {
            monday: 1,
            tuesday: 2,
            wednesday: 3,
            thursday: 4,
            friday: 5,
            saturday: 6,
            sunday: 7,
        }
    }

    mod default {
        use super::*;

        #[test]
        fn should_work_eight_hours_on_workdays() {
            let w = WorkPerDay::default();

            for minutes in [w.monday, w.tuesday, w.wednesday, w.thursday, w.friday] {
                assert_eq!(8 * 60, minutes);
            }
        }

        #[test]
        fn should_not_work_on_the_weekend() {
            let w = WorkPerDay::default();

            assert_eq!(0, w.saturday);
            assert_eq!(0, w.sunday);
        }
    }

    mod config_value {
        use super::*;

        #[test]
        fn should_convert_to_config_table() {
            let table = Value::from(varied()).into_table().unwrap();

            assert_eq!(7, table.len());
            for (day, minutes) in [
                ("monday", 1),
                ("tuesday", 2),
                ("wednesday", 3),
                ("thursday", 4),
                ("friday", 5),
                ("saturday", 6),
                ("sunday", 7),
            ] {
                assert_eq!(minutes, table[day].clone().into_int().unwrap(), "{day}");
            }
        }

        #[test]
        fn should_convert_default_to_config_table() {
            let table = Value::from(WorkPerDay::default()).into_table().unwrap();

            assert_eq!(480, table["friday"].clone().into_int().unwrap());
            assert_eq!(0, table["saturday"].clone().into_int().unwrap());
        }
    }

    mod json {
        use super::*;

        #[test]
        fn should_serialize_each_weekday() {
            assert_eq!(
                "{\"monday\":1,\"tuesday\":2,\"wednesday\":3,\"thursday\":4,\"friday\":5,\"saturday\":6,\"sunday\":7}",
                serde_json::to_string(&varied()).unwrap()
            );
        }

        #[test]
        fn should_roundtrip() {
            let json = serde_json::to_string(&varied()).unwrap();

            assert_eq!(varied(), serde_json::from_str::<WorkPerDay>(&json).unwrap());
        }

        #[test]
        fn should_roundtrip_default() {
            let json = serde_json::to_string(&WorkPerDay::default()).unwrap();

            assert_eq!(WorkPerDay::default(), serde_json::from_str::<WorkPerDay>(&json).unwrap());
        }

        #[test]
        fn should_reject_missing_weekday() {
            assert!(serde_json::from_str::<WorkPerDay>("{\"monday\":1}").is_err());
        }
    }

    mod minutes_on {
        use super::*;

        /// Monday 2022-02-07 to Sunday 2022-02-13.
        fn day(offset: u32) -> NaiveDate {
            NaiveDate::from_ymd_opt(2022, 2, 7 + offset).unwrap()
        }

        #[test]
        fn should_pick_the_weekday_of_the_date() {
            let w = varied();

            for offset in 0..7 {
                assert_eq!(offset as u16 + 1, w.minutes_on(day(offset)), "{}", day(offset));
            }
        }

        #[test]
        fn should_accept_a_date_time() {
            let w = varied();
            let friday = Utc.with_ymd_and_hms(2022, 2, 11, 23, 59, 0).unwrap();

            assert_eq!(5, w.minutes_on(friday));
        }

        #[test]
        fn should_repeat_every_week() {
            let w = varied();

            assert_eq!(w.minutes_on(day(2)), w.minutes_on(day(2) + chrono::Duration::weeks(10)));
        }

        #[test]
        fn should_use_default_work_time() {
            let w = WorkPerDay::default();

            assert_eq!(480, w.minutes_on(day(0)));
            assert_eq!(0, w.minutes_on(day(5)));
            assert_eq!(0, w.minutes_on(day(6)));
        }
    }
}
