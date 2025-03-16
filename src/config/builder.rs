use std::path::PathBuf;

use config::{ builder::DefaultState, Config, ConfigBuilder, FileFormat };
use nameof::name_of;

use crate::config::Configuration;

use super::ConfigurationError;

#[derive(Clone)]
pub struct ConfigurationBuilder {
    state: ConfigBuilder<DefaultState>,
}

impl ConfigurationBuilder {
    pub fn new() -> Result<Self, ConfigurationError> {
        let defaults = Configuration::default();
        let state = Config::builder()
            .set_default(name_of!(folder in Configuration), defaults.folder)?
            .set_default(name_of!(threshold_limits in Configuration), defaults.threshold_limits)?
            .set_default(name_of!(limits in Configuration), defaults.limits)?
            .set_default(name_of!(workperday in Configuration), defaults.workperday)?;
        Ok(ConfigurationBuilder { state })
    }
    pub fn add_json_source(&self, file: &PathBuf) -> Result<Self, ConfigurationError> {
        let state = self.state
            .clone()
            .add_source(config::File::new(file.to_str().unwrap(), FileFormat::Json));
        Ok(ConfigurationBuilder { state })
    }

    pub fn build(&self) -> Result<Configuration, ConfigurationError> {
        let s = self.state.clone().build()?;
        log::debug!("used configuration: {:?}", s);
        Ok(s.try_deserialize()?)
    }
}
