use std::{ env, fs::{self, OpenOptions}, io::Write, path::PathBuf };

use config::{builder::DefaultState, Config, ConfigBuilder, FileFormat};
use nameof::name_of;
use serde::{ Deserialize, Serialize };

use super::{ BreakThreshold, ConfigurationError, WorktimePerDay };

pub(super) const CONFIG_FILE: &str = "trackrs.conf";

#[derive(Serialize)]
pub struct ReqSettings {
    pub folder: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(unused)]
pub struct Configuration {
    pub folder: String,
    pub threshold_limits: u8,
    pub limits: Vec<BreakThreshold>,
    pub workperday: WorktimePerDay,
}

impl Default for Configuration {
    fn default() -> Self {
        Self {
            folder: dirs::home_dir().unwrap_or(env::temp_dir()).to_str().unwrap().to_string(),
            threshold_limits: 1,
            limits: vec!(),
            workperday: Default::default(),
        }
    }
}

impl Configuration {
    pub fn builder() -> ConfigurationBuilder {
        ConfigurationBuilder::new()
    }

    pub fn file() -> PathBuf {
        let d = match env::var("RUST_TEST") {
            Ok(_) => env::current_dir().unwrap(),
            Err(_) => dirs::config_dir().unwrap(),
        };
        d.join(CONFIG_FILE)
    }
}


#[derive(Clone)]
pub struct ConfigurationBuilder {
    state: ConfigBuilder<DefaultState>,
    source_file: Option<PathBuf>,
}

impl ConfigurationBuilder {
    pub fn new() -> Self {
        ConfigurationBuilder { state: Config::builder(), source_file: None }
    }

    pub fn add_defaults(&self) -> Result<Self, ConfigurationError> {
        let defaults = Configuration::default();
        let state = self.state
            .clone()
            .set_default(name_of!(folder in Configuration), defaults.folder)?
            .set_default(name_of!(threshold_limits in Configuration), defaults.threshold_limits)?
            .set_default(name_of!(limits in Configuration), defaults.limits)?
            .set_default(name_of!(workperday in Configuration), defaults.workperday)?;
        Ok(ConfigurationBuilder { state, source_file: self.source_file.clone() })
    }

    pub fn add_json_source(&self, file: &PathBuf) -> Result<Self, ConfigurationError> {
        if !file.exists() {
            let mut f = fs::File::create(file)?;
            f.write_all(b"{}")?;
        }

        let state = self.state
            .clone()
            .add_source(config::File::new(file.to_str().unwrap(), FileFormat::Json));
        Ok(ConfigurationBuilder { state, source_file: Some(file.clone()) })
    }

    pub fn build(&self) -> Result<Configuration, ConfigurationError> {
        let s = self.state.clone().build()?;
        log::debug!("used configuration: {:?}", s);
        let configuration: Configuration = s.try_deserialize()?;

        if self.source_file.is_some() {
            log::debug!("saving configuration to '{:?}'", self.source_file.as_ref().unwrap());
            let mut w = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .truncate(false)
                .open(self.source_file.as_ref().unwrap())?;
            serde_json::to_writer(&w, &configuration)?;
            w.flush()?;
        }
        Ok(configuration)
    }
}


#[cfg(test)]
mod tests {
    use std::{ fs::File, io::Read };

    use crate::config::CONFIG_FILE;

    use super::Configuration;

    #[test]
    fn should_return_user_folder() {
        std::env::remove_var("RUST_TEST");
        let path = Configuration::file();
        assert!(
            path.starts_with(dirs::config_dir().unwrap()),
            "path: '{:?}', RUST_TEST: '{:?}'",
            path,
            std::env::var("RUST_TEST")
        );
        assert!(path.ends_with(CONFIG_FILE))
    }

    #[test]
    fn should_return_current_folder() {
        std::env::set_var("RUST_TEST", "true");
        let path = Configuration::file();
        assert!(
            path.starts_with(std::env::current_dir().unwrap()),
            "path: '{:?}', RUST_TEST: '{:?}'",
            path,
            std::env::var("RUST_TEST")
        );
        assert!(path.ends_with(CONFIG_FILE))
    }

    #[test]
    fn should_not_build_empty() {
        let config = Configuration::builder().build();
        assert!(config.is_err())
    }

    #[test]
    fn should_build_with_defaults() {
        let defaults = Configuration::default();
        let config = Configuration::builder().add_defaults().unwrap().build();
        assert!(config.is_ok());
        assert_eq!(config.unwrap().folder, defaults.folder);
    }

    #[test]
    fn should_not_build_with_file_missing_properties() {
        let configfile = tempfile::tempdir().unwrap().into_path().join(CONFIG_FILE);
        assert!(!configfile.exists());

        let config = Configuration::builder().add_json_source(&configfile).unwrap().build();
        assert!(config.is_err());
        assert!(configfile.exists());
    }

    #[test]
    fn should_build_with_defaults_and_file() {
        let defaults = Configuration::default();

        let configfile = tempfile::tempdir().unwrap().into_path().join(CONFIG_FILE);
        assert!(!configfile.exists());

        let config = Configuration::builder()
            .add_defaults()
            .unwrap()
            .add_json_source(&configfile)
            .unwrap()
            .build();
        assert!(config.is_ok(), "{:?}", config.unwrap_err());
        assert_eq!(config.unwrap().folder, defaults.folder);
        assert!(configfile.exists());

        let mut file = File::open(&configfile).unwrap();
        let mut buf = vec![];
        file.read_to_end(&mut buf).unwrap();
        let content = String::from_utf8(buf).unwrap();
        assert_ne!(content, "{}", "configuration was saved")
    }
}
