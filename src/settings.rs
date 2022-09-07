use std::{env, fs::OpenOptions, path::Path};

use chrono::{Date, DateTime, Datelike, Local};
use config::{Config, ConfigError, File, FileFormat, Map, Value, ValueKind};
use serde::Serialize;
use serde_derive::Deserialize;

use crate::TrackerError;

#[derive(Serialize)]
#[allow(unused)]
pub struct ReqSettings {
    pub folder: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub file: String,
    pub folder: String,
    pub threshold_limits: u8,
    pub limits: Vec<BreakLimit>,
    pub workperday: WorkPerDayInMinutes,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[allow(unused)]
pub struct BreakLimit {
    pub start: u8,
    pub minutes: u8,
}

impl From<BreakLimit> for config::Value {
    fn from(l: BreakLimit) -> Self {
        let mut m = Map::new();
        m.insert(
            "start".to_owned(),
            Value::new(Some(&"start".to_owned()), ValueKind::U64(l.start.into())),
        );
        m.insert(
            "minutes".to_owned(),
            Value::new(
                Some(&"minutes".to_owned()),
                ValueKind::U64(l.minutes.into()),
            ),
        );
        Value::new(Some(&"BreakLimit".to_string()), ValueKind::Table(m))
    }
}

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

impl Default for Settings {
    fn default() -> Self {
        Self {
            file: Default::default(),
            folder: dirs::home_dir().unwrap().to_str().unwrap().to_owned(),
            threshold_limits: 1,
            limits: [].to_vec(),
            workperday: Default::default(),
        }
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

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // env::set_var("RUST_TEST", "true")
        let d = match env::var("RUST_TEST") {
            Ok(_) => env::current_dir().unwrap(),
            Err(_) => dirs::home_dir().unwrap(),
        };

        Settings::build(&d)
    }

    fn build(base: &Path) -> Result<Self, ConfigError> {
        let d = Settings::default();
        let f = base.join(".trackrs");
        Settings::assert_created(&f).unwrap();
        let s = Config::builder()
            .set_default("file", f.to_str().unwrap())?
            .set_default("threshold_limits", d.threshold_limits)?
            .set_default("limits", d.limits)?
            .set_default("workperday", d.workperday)?
            .add_source(File::new(f.to_str().unwrap(), FileFormat::Json))
            .build()?;
        log::debug!("configuration: {:?}", s);
        s.try_deserialize()
    }

    fn assert_created(file_path: &Path) -> Result<(), TrackerError> {
        if !file_path.exists() {
            let w = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .truncate(false)
                .open(file_path)?;
            serde_json::to_writer_pretty(w, &Settings::required_fields())?;
        }
        Ok(())
    }

    fn required_fields() -> ReqSettings {
        let d = Settings::default();
        ReqSettings { folder: d.folder }
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

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

    use crate::{BreakLimit, Settings, TrackerError, WorkPerDayInMinutes};

    mod settings {

        use super::*;

        #[test]
        fn should_create() -> Result<(), TrackerError> {
            let temp_dir = tempfile::tempdir()?;

            let settings = Settings::build(temp_dir.path()).unwrap();
            assert!(&temp_dir.into_path().join(".trackrs").exists());
            assert_eq!(dirs::home_dir().unwrap().to_str().unwrap(), settings.folder);
            assert_eq!(1, settings.threshold_limits);
            assert_eq!(0, settings.limits.len());
            assert_eq!(8 * 60, settings.workperday.wednesday);
            assert_eq!(0, settings.workperday.saturday);
            Ok(())
        }

        #[test]
        fn should_read() -> Result<(), TrackerError> {
            let expected_settings = Settings {
                file: "file".to_owned(),
                folder: "/temp/dir".to_owned(),
                threshold_limits: 25,
                limits: [
                    BreakLimit {
                        start: 3,
                        minutes: 40,
                    },
                    BreakLimit {
                        start: 2,
                        minutes: 15,
                    },
                ]
                .to_vec(),
                workperday: WorkPerDayInMinutes {
                    monday: 6 * 60,
                    tuesday: 7 * 60,
                    wednesday: 8 * 60,
                    thursday: 6 * 60,
                    friday: 4 * 60,
                    saturday: 0,
                    sunday: 0,
                },
            };

            let temp_dir = tempfile::tempdir()?;
            let f = temp_dir.path().join(".trackrs");

            let w = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .truncate(false)
                .open(f)?;
            serde_json::to_writer(w, &expected_settings)?;

            let settings = Settings::build(temp_dir.path()).unwrap();

            assert_eq!(expected_settings.folder, settings.folder);
            assert_eq!(expected_settings.limits.len(), settings.limits.len());
            assert_eq!(
                expected_settings.threshold_limits,
                settings.threshold_limits
            );
            assert!(settings.limits.contains(&expected_settings.limits[0]));
            assert!(settings.limits.contains(&expected_settings.limits[1]));
            Ok(())
        }
    }
}
