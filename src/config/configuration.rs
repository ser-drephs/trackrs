use std::fs::OpenOptions;

use serde::{ Deserialize, Serialize };

use crate::config::ConfigurationFile;

use super::{
    break_limit::BreakLimit,
    work_per_day::WorkPerDayInMinutes,
    ConfigurationBuilder,
    ConfigurationError,
};

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
    pub fn builder() -> ConfigurationBuilder {
        ConfigurationBuilder::new().unwrap()
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
        let config = Configuration::builder().build();
        assert!(config.is_ok())
    }

    #[test]
    fn should_set_defaults() {
        let defaults = Configuration::default();
        let config = Configuration::builder().build();
        assert!(config.is_ok());
        assert_eq!(config.unwrap().folder, defaults.folder);
    }

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
