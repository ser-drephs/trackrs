use std::{env, fs::OpenOptions, path::Path};

use chrono::{Date, DateTime, Datelike, Local};
use config::{Config, ConfigError, File, FileFormat, Map, Value, ValueKind};
use serde::Serialize;
use serde_derive::Deserialize;

use crate::TrackerError;






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
                        start: 3 * 60,
                        minutes: 40,
                    },
                    BreakLimit {
                        start: 2 * 60,
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
