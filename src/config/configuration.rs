use std::fs::OpenOptions;

use config::{ Config, FileFormat };
use nameof::name_of;
use serde::{ Deserialize, Serialize };

use crate::config::ConfigurationFile;

use super::{ break_limit::BreakLimit, work_per_day::WorkPerDayInMinutes, ConfigurationError };

#[derive(Serialize)]
#[allow(unused)]
pub struct ReqSettings {
    pub folder: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct Configuration {
    pub folder: String,
    pub threshold_limits: u8,
    pub limits: Vec<BreakLimit>,
    pub workperday: WorkPerDayInMinutes,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            folder: dirs::home_dir().unwrap().to_str().unwrap().to_owned(),
            threshold_limits: 1,
            limits: vec!(),
            workperday: Default::default(),
        }
    }
}

impl Configuration {
    pub fn new() -> Result<Self, ConfigurationError> {
        let file = ConfigurationFile::file();
        let defaults = Configuration::default();

        let s = Config::builder()
            .set_default(name_of!(folder in Configuration), defaults.folder)?
            .set_default(name_of!(threshold_limits in Configuration), defaults.threshold_limits)?
            .set_default(name_of!(limits in Configuration), defaults.limits)?
            .set_default(name_of!(workperday in Configuration), defaults.workperday)?
            .add_source(config::File::new(file.to_str().unwrap(), FileFormat::Json))
            .build()?;

        log::debug!("used configuration: {:?}", s);
        Ok(s.try_deserialize()?)
    }

    pub fn save(&self) -> Result<(), ConfigurationError> {
        let file = ConfigurationFile::file();

        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(file)?;
        Ok(serde_json::to_writer(w, &self)?)
    }
}

#[cfg(test)]
mod tests {
    use super::Configuration;

    #[test]
    fn should_error_no_file() {
        let config = Configuration::new();
        assert!(config.is_err())
    }

    // #[test]
    // fn should_set_defaults() {
    //     // TODO: add test context with file exists
    //     crate::test::setup();
    //     crate::config::ConfigurationFile::verify().unwrap();
    //     let defaults = Configuration::default();
    //     let config = Configuration::new();
    //     assert!(config.is_ok());
    //     assert_eq!(config.unwrap().folder, defaults.folder);
    // }

    //  #[test]
    // fn should_save_defaults() {
    //     // TODO: add test context with file exists
    //     crate::test::setup();
    //     crate::config::ConfigurationFile::verify().unwrap();
    //     let config = Configuration::new();
    //     assert!(config.is_ok());
    //     let save = config.unwrap().save();
    //     assert!(save.is_ok());
    //     let file = crate::config::ConfigurationFile::file();
    //     assert!(file.exists());
    //     assert!(file.metadata().unwrap().len() > 100)
    // }
}
