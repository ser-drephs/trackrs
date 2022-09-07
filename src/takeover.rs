use std::{
    env,
    fs::{File, OpenOptions},
    path::PathBuf,
};

use serde::{Deserialize, Serialize};

use crate::TrackerError;

#[derive(Default, Clone, Deserialize, Serialize, Debug)]
pub struct Takeover {
    pub minutes: Option<u16>,
}

impl Takeover {
    pub fn builder() -> TakeoverBuilder {
        TakeoverBuilder::default()
    }
}

#[derive(Default)]
pub struct TakeoverBuilder {
    file: Option<PathBuf>,
}

impl TakeoverBuilder {
    pub fn file(&mut self) -> &mut Self {
        let d = match env::var("RUST_TEST") {
            Ok(_) => env::current_dir().unwrap_or_default(),
            Err(_) => dirs::home_dir().unwrap_or_default(),
        };
        let f = d.join(".trackrs-takeover");
        self.file = Some(f);
        self
    }

    pub fn set(&self, minutes: u16) -> Result<Takeover, TrackerError> {
        if self.file.is_none() {
            Err(TrackerError::TakeoverSetError {
                message: "takeover file not set".to_owned(),
            })
        } else if minutes == 0 {
            log::warn!("won't take over less than 0 minutes");
            Ok(Takeover::default())
        } else {
            let t = Takeover {
                minutes: Some(minutes),
            };

            log::debug!("takeover {} minutes next time", t.minutes.unwrap());
            let w = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .truncate(false)
                .open(self.file.as_ref().unwrap())?;
            serde_json::to_writer(w, &t)?;
            Ok(t)
        }
    }

    pub fn get(&self) -> Result<Takeover, TrackerError> {
        if self.file.is_none() {
            Err(TrackerError::TakeoverGetError {
                message: "takeover file not set".to_owned(),
            })
        } else {
            let f = self.file.as_ref().unwrap();
            let mut t = Takeover::default();
            if f.exists() {
                log::debug!("takeover was requested");
                let f = File::open(&f)?;
                t = serde_json::from_reader(f)?;
                if t.minutes.is_none() || t.minutes.as_ref().unwrap() <= &0 {
                    t.minutes = None;
                }
            } else {
                log::debug!("no takeover requested");
            }
            Ok(t)
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::TrackerError;

    use std::env;
    use std::fs::File;

    use std::io::Write;

    use crate::Takeover;

    use std::env::set_current_dir;

    use std::fs;

    fn logger() {
        // std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn test_env() {
        env::set_var("RUST_TEST", "true");
    }

    mod get {

        use super::*;

        #[test]
        fn file_not_exists() -> Result<(), TrackerError> {
            logger();
            test_env();
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let mut b = Takeover::builder();
            let t = b.file().get()?;
            assert!(t.minutes.is_none());
            Ok(())
        }

        #[test]
        fn file_contains_time() -> Result<(), TrackerError> {
            logger();
            test_env();
            let file_content = "{\"minutes\":15}";
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut b = Takeover::builder();
            b.file = Some(time_file);
            let t = b.get()?;

            assert!(t.minutes.is_some());
            assert_eq!(&15, t.minutes.as_ref().unwrap());

            Ok(())
        }

        #[test]
        fn file_contains_zero_time() -> Result<(), TrackerError> {
            logger();
            test_env();
            let file_content = "{\"minutes\":0}";
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut b = Takeover::builder();
            b.file = Some(time_file);
            let t = b.get()?;

            assert!(t.minutes.is_none());
            Ok(())
        }
    }

    mod set {

        use super::*;

        #[test]
        fn create_takeover_file() -> Result<(), TrackerError> {
            logger();
            test_env();
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut b = Takeover::builder();
            b.file = Some(time_file.to_owned());
            let t = b.set(25)?;

            let exp_content = "{\"minutes\":25}";
            let act_content = fs::read_to_string(time_file)?;

            assert_eq!(exp_content, act_content);
            assert!(t.minutes.is_some());
            assert_eq!(25, t.minutes.unwrap());
            Ok(())
        }

        #[test]
        fn create_empty_takeover() -> Result<(), TrackerError> {
            logger();
            test_env();
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut b = Takeover::builder();
            let t = b.file().set(0)?;

            assert!(!time_file.exists());
            assert!(t.minutes.is_none());
            Ok(())
        }
    }
}
