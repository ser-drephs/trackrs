use crate::{TrackerError};
use config::{Config, ConfigError, File, FileFormat};
use serde::Serialize;
use serde_derive::Deserialize;
use std::{env, fs::OpenOptions, path::Path};

use super::{WorkPerDayInMinutes, BreakLimit};

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

    pub fn build(base: &Path) -> Result<Self, ConfigError> {
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

    use crate::{Settings, TrackerError};

    mod settings {


    }
}
