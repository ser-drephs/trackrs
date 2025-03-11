use std::{ env, path::PathBuf };

use config::{ File, FileFormat };

use super::{ ConfigurationProvider, ConfigurationProviderError };

const CONFIG_FILE: &str = "trackrs.conf";

pub struct JsonConfigurationProvider {
    file: PathBuf,
}

impl JsonConfigurationProvider {
    pub fn new() -> Self {
        let d = match env::var("RUST_TEST") {
            Ok(_) => env::current_dir().unwrap(),
            Err(_) => dirs::config_dir().unwrap(),
        };
        let file = d.join(CONFIG_FILE);
        JsonConfigurationProvider { file }
    }
}

impl ConfigurationProvider for JsonConfigurationProvider {
    fn source(&self) -> Result<Box<dyn config::Source>, ConfigurationProviderError> {
        let filesource = File::new(self.file.to_str().unwrap(), FileFormat::Json);
        Ok(filesource)
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
