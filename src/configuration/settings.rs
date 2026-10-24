use std::{ env, fs::OpenOptions, path::Path };

use chrono::Duration;
use config::{ Config, ConfigError, File, FileFormat };
use serde::Serialize;
use serde_derive::Deserialize;

use crate::{ BreakLimit, TrackerError, WorkPerDay };

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
    pub workperday: WorkPerDay,
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

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        // env::set_var("RUST_TEST", "true")
        let d = match env::var("RUST_TEST") {
            Ok(_) => env::current_dir().unwrap(),
            Err(_) => dirs::home_dir().unwrap(),
        };

        Settings::build(&d)
    }

    /// Break to take for the given worktime, based on the highest reached break limit.
    pub fn break_for(&self, worktime: Duration) -> Duration {
        self.limits
            .iter()
            .filter(|l| worktime >= Duration::minutes(l.start.into()))
            .max_by_key(|l| l.start)
            .map_or(Duration::zero(), |l| Duration::minutes(l.minutes.into()))
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

#[cfg(test)]
mod tests {
    use std::fs::OpenOptions;

    use crate::{ BreakLimit, Settings, TrackerError, WorkPerDay };

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
                    start: 3 * 60,
                    minutes: 40,
                },
                BreakLimit {
                    start: 2 * 60,
                    minutes: 15,
                },
            ].to_vec(),
            workperday: WorkPerDay {
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

        let w = OpenOptions::new().create(true).write(true).append(false).truncate(false).open(f)?;
        serde_json::to_writer(w, &expected_settings)?;

        let settings = Settings::build(temp_dir.path()).unwrap();

        assert_eq!(expected_settings.folder, settings.folder);
        assert_eq!(expected_settings.limits.len(), settings.limits.len());
        assert_eq!(expected_settings.threshold_limits, settings.threshold_limits);
        assert!(settings.limits.contains(&expected_settings.limits[0]));
        assert!(settings.limits.contains(&expected_settings.limits[1]));
        Ok(())
    }

    mod break_for {
        use chrono::Duration;

        use super::*;

        fn settings(limits: &[(u16, u8)]) -> Settings {
            Settings {
                limits: limits
                    .iter()
                    .map(|(start, minutes)| BreakLimit { start: *start, minutes: *minutes })
                    .collect(),
                ..Default::default()
            }
        }

        fn break_minutes(settings: &Settings, worktime_minutes: i64) -> i64 {
            settings.break_for(Duration::minutes(worktime_minutes)).num_minutes()
        }

        #[test]
        fn should_not_break_without_limits() {
            assert_eq!(0, break_minutes(&settings(&[]), 600));
        }

        #[test]
        fn should_not_break_below_first_limit() {
            let s = settings(&[(360, 30), (540, 45)]);

            assert_eq!(0, break_minutes(&s, 0));
            assert_eq!(0, break_minutes(&s, 359));
        }

        #[test]
        fn should_break_from_the_start_of_a_limit_on() {
            let s = settings(&[(360, 30), (540, 45)]);

            assert_eq!(30, break_minutes(&s, 360));
            assert_eq!(30, break_minutes(&s, 539));
            assert_eq!(45, break_minutes(&s, 540));
        }

        #[test]
        fn should_use_highest_reached_limit() {
            let s = settings(&[(360, 30), (540, 45), (720, 60)]);

            assert_eq!(60, break_minutes(&s, 721));
            assert_eq!(60, break_minutes(&s, 2000));
        }

        #[test]
        fn should_ignore_order_of_limits() {
            let s = settings(&[(720, 60), (360, 30), (540, 45)]);

            assert_eq!(30, break_minutes(&s, 400));
            assert_eq!(45, break_minutes(&s, 600));
            assert_eq!(60, break_minutes(&s, 800));
        }

        #[test]
        fn should_prefer_highest_start_over_longer_break() {
            // The highest reached limit wins, even if it has a shorter break than a lower one.
            let s = settings(&[(360, 45), (540, 30)]);

            assert_eq!(30, break_minutes(&s, 600));
        }
    }
}
