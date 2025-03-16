use std::{ env, fs, io::Write, path::PathBuf };
use super::ConfigurationError;

const CONFIG_FILE: &str = "trackrs.conf";

pub struct ConfigurationFile {}

impl ConfigurationFile {
    pub fn file() -> PathBuf {
        let d = match env::var("RUST_TEST") {
            Ok(_) => env::current_dir().unwrap(),
            Err(_) => dirs::config_dir().unwrap(),
        };
        d.join(CONFIG_FILE)
    }

    pub fn verify() -> Result<(), ConfigurationError> {
        let file = ConfigurationFile::file();
        if !file.exists() {
            let mut f = fs::File::create(file)?;
            f.write_all(b"{}")?;
        }
        Ok(())
    }
}
