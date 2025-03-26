use chrono::{ Duration, Weekday };
use config::{ Map, Value, ValueKind };
use serde::{ Deserialize, Serialize };

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct WorktimePerDay {
    monday: u16,
    tuesday: u16,
    wednesday: u16,
    thursday: u16,
    friday: u16,
    saturday: u16,
    sunday: u16,
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

impl WorktimePerDay {
    pub fn into_u16(&self, weekday: Weekday) -> &u16 {
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

    pub fn into_duration(&self, weekday: Weekday) -> Duration {
        let minutes = self.into_u16(weekday);
        Duration::minutes((*minutes).into())
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
        let for_tuesday = defaults.into_u16(tuesday.weekday());
        assert_eq!(&480, for_tuesday);

        let saturday = Local.with_ymd_and_hms(2025, 2, 22, 8, 0, 0).unwrap().to_utc();
        let for_saturday = defaults.into_u16(saturday.weekday());
        assert_eq!(&0, for_saturday)
    }

    #[test]
    fn should_weekday_into_duration() {
        let defaults = WorktimePerDay::default();
        let tuesday = Local.with_ymd_and_hms(2025, 2, 18, 8, 0, 0).unwrap().to_utc();
        let for_tuesday = defaults.into_duration(tuesday.weekday());
        assert_eq!(Duration::minutes(480), for_tuesday);

        let saturday = Local.with_ymd_and_hms(2025, 2, 22, 8, 0, 0).unwrap().to_utc();
        let for_saturday = defaults.into_duration(saturday.weekday());
        assert_eq!(Duration::minutes(0), for_saturday)
    }
}
