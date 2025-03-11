use std::{ any::Any, env, fs::OpenOptions, ops::Deref, path::Path };

use config::{ Config, File, FileFormat, Source };
use serde::{ Deserialize, Serialize };

use super::{
    break_limit::BreakLimit,
    work_per_day::WorkPerDayInMinutes,
    ConfigurationError,
    ConfigurationProvider,
};

#[derive(Serialize)]
#[allow(unused)]
pub struct ReqSettings {
    pub folder: String,
}

#[derive(Debug, Deserialize, Serialize)]
// Clone
#[allow(unused)]
pub struct Configuration {
    pub file: String,
    pub folder: String,
    pub threshold_limits: u8,
    pub limits: Vec<BreakLimit>,
    pub workperday: WorkPerDayInMinutes,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            file: Default::default(),
            folder: dirs::home_dir().unwrap().to_str().unwrap().to_owned(),
            threshold_limits: 1,
            limits: vec!(),
            workperday: Default::default(),
        }
    }
}

impl Configuration {
    pub fn new<C: ConfigurationProvider>(config: &C) -> Result<Self, ConfigurationError> {
        let defaults = Configuration::default();
        let source = config.source()?.as_ref();

        // let a = source.as_ref();
        // let source_ref = &source;

        let s = Config::builder()
            .set_default("threshold_limits", defaults.threshold_limits)?
            .set_default("limits", defaults.limits)?
            .set_default("workperday", defaults.workperday)?
            .add_source(source)
            .build()?;
        log::debug!("configuration: {:?}", s);
        Ok(s.try_deserialize()?)
    }

    // fn assert_created(file_path: &Path) -> Result<(), ConfigurationError> {
    //     if !file_path.exists() {
    //         let w = OpenOptions::new()
    //             .create(true)
    //             .write(true)
    //             .append(false)
    //             .truncate(false)
    //             .open(file_path)?;
    //         serde_json::to_writer_pretty(w, &Configuration::required_fields())?;
    //     }
    //     Ok(())
    // }

    fn required_fields() -> ReqSettings {
        let d = Configuration::default();
        ReqSettings { folder: d.folder }
    }
}
