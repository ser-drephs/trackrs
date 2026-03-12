use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use config::{ Map, Value, ValueKind };
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorktimePerDay {
    // TODO: make fields private again
    pub monday: u16,
    pub tuesday: u16,
    pub wednesday: u16,
    pub thursday: u16,
    pub friday: u16,
    pub saturday: u16,
    pub sunday: u16,
}

impl WorktimePerDay {
    pub fn get_by_workday(&self, weekday: Weekday) -> &u16 {
        match weekday {
            Weekday::Mon => &self.monday,
            Weekday::Tue => &self.tuesday,
            Weekday::Wed => &self.wednesday,
            Weekday::Thu => &self.thursday,
            Weekday::Fri => &self.friday,
            Weekday::Sat => &self.saturday,
            Weekday::Sun => &self.sunday,
        }
    }

    pub fn get_duration_by_weekday(&self, weekday: Weekday) -> Duration {
        let minutes = self.get_by_workday(weekday);
        Duration::minutes((*minutes).into())
    }

    pub fn get_duration_by_date(&self, date: DateTime<Utc>) -> Duration {
        let minutes = self.get_by_workday(date.weekday());
        Duration::minutes((*minutes).into())
    }
}

impl From<WorktimePerDay> for config::Value {
    fn from(w: WorktimePerDay) -> Self {
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

impl Default for WorktimePerDay {
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

#[cfg(test)]
mod tests {
    use chrono::{ Datelike, Duration, Local, TimeZone };
    use config::{ Config, File, FileFormat };
    use nameof::name_of;
    use serde::Deserialize;

    use super::WorktimePerDay;

    #[derive(Debug, Deserialize)]
    struct Dummy {
        workperday: WorktimePerDay,
    }

    #[test]
    fn should_deserialize_config() {
        let settings = Config::builder()
            .add_source(
                File::from_str(
                    "{\"monday\":200,\"tuesday\":0,\"wednesday\":20,\"thursday\":0,\"friday\":0,\"saturday\":0,\"sunday\":0}",
                    FileFormat::Json
                )
            )
            .build()
            .unwrap();

        let res = settings.try_deserialize::<WorktimePerDay>();
        assert!(res.is_ok(), "{:?}", res.err());
        let work = res.unwrap();
        assert_eq!(work.monday, 200);
        assert_eq!(work.friday, 0)
    }

    #[test]
    fn should_accept_default_config() {
        let settings = Config::builder()
            .set_default(name_of!(workperday in Dummy), WorktimePerDay::default())
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
        let defaults = WorktimePerDay::default();
        let tuesday = Local.with_ymd_and_hms(2025, 2, 18, 8, 0, 0).unwrap().to_utc();
        let for_tuesday = defaults.get_by_workday(tuesday.weekday());
        assert_eq!(&480, for_tuesday);

        let saturday = Local.with_ymd_and_hms(2025, 2, 22, 8, 0, 0).unwrap().to_utc();
        let for_saturday = defaults.get_by_workday(saturday.weekday());
        assert_eq!(&0, for_saturday)
    }

    #[test]
    fn should_weekday_into_duration() {
        let defaults = WorktimePerDay::default();
        let tuesday = Local.with_ymd_and_hms(2025, 2, 18, 8, 0, 0).unwrap().to_utc();
        let for_tuesday = defaults.get_duration_by_weekday(tuesday.weekday());
        assert_eq!(Duration::minutes(480), for_tuesday);

        let saturday = Local.with_ymd_and_hms(2025, 2, 22, 8, 0, 0).unwrap().to_utc();
        let for_saturday = defaults.get_duration_by_weekday(saturday.weekday());
        assert_eq!(Duration::minutes(0), for_saturday)
    }
}
