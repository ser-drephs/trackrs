use config::{ Map, Value, ValueKind };
use serde::{ Deserialize, Serialize };
use time::{ Duration, Weekday };

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
        Value::new(Some(&"workperday".to_owned()), ValueKind::Table(m))
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
    pub fn into_u16(&self, day: Weekday) -> &u16 {
        let expected = match day {
            Weekday::Monday => &self.monday,
            Weekday::Tuesday => &self.tuesday,
            Weekday::Wednesday => &self.wednesday,
            Weekday::Thursday => &self.thursday,
            Weekday::Friday => &self.friday,
            Weekday::Saturday => &self.saturday,
            Weekday::Sunday => &self.sunday,
        };
        log::debug!("work per day for '{}': {} ", day, expected);
        expected
    }

    pub fn into_duration(&self, day: Weekday) -> Duration {
        let minutes = self.into_u16(day);
        Duration::minutes((*minutes).into())
    }
}

#[cfg(test)]
mod tests {
    use config::{ Config, File, FileFormat };
    use serde::Deserialize;
    use time::{ macros::date, Duration };

    use crate::config::work_per_day::WorkPerDayInMinutes;

    #[derive(Debug, Deserialize)]
    struct Dummy {
        workperday: WorkPerDayInMinutes,
    }

    #[test]
    fn should_not_deserialize_with_missing() {
        let settings = Config::builder()
            .add_source(
                File::from_str(
                    "{\"monday\":200,\"tuesday\":0,\"thursday\":0,\"wednesday\":20,\"friday\":0,\"saturday\":0,\"sunday\":0}",
                    FileFormat::Json
                )
            )
            .build()
            .unwrap();

        let res = settings.try_deserialize::<WorkPerDayInMinutes>();
        assert!(res.is_ok(), "{:?}", res.err());
        let work = res.unwrap();
        assert_eq!(work.monday, 200);
        assert_eq!(work.friday, 0)
    }

    #[test]
    fn should_accept_default_config() {
        let settings = Config::builder()
            .set_default("workperday", WorkPerDayInMinutes::default())
            .unwrap()
            .add_source(File::from_str("{}", FileFormat::Json))
            .build()
            .unwrap();

        let res = settings.try_deserialize::<Dummy>();
        assert!(res.is_ok(), "{:?}", res.err());
        let dummy = res.unwrap();
        assert_eq!(dummy.workperday.monday, 480);
        assert_eq!(dummy.workperday.friday, 480)
    }

    #[test]
    fn should_weekday_into_number() {
        let defaults = WorkPerDayInMinutes::default();
        let tuesday = date!(2025 - 02 - 18);
        let for_tuesday = defaults.into_u16(tuesday.weekday());
        assert_eq!(&480, for_tuesday);
        let saturday = date!(2025 - 02 - 22);
        let for_saturday = defaults.into_u16(saturday.weekday());
        assert_eq!(&0, for_saturday)
    }

    #[test]
    fn should_weekday_into_duration() {
        let defaults = WorkPerDayInMinutes::default();
        let tuesday = date!(2025 - 02 - 18);
        let for_tuesday = defaults.into_duration(tuesday.weekday());
        assert_eq!(Duration::minutes(480), for_tuesday);
        let saturday = date!(2025 - 02 - 22);
        let for_saturday = defaults.into_duration(saturday.weekday());
        assert_eq!(Duration::minutes(0), for_saturday)
    }
}
