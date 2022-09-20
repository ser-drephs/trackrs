use crate::TimeDataError;
use serde::{Deserialize, Serialize};
use std::{
    env,
    fs::{File, OpenOptions},
    path::PathBuf,
};

type Result = std::result::Result<Takeover, TimeDataError>;

#[derive(Default, Clone, Deserialize, Serialize, Debug)]
pub struct Takeover {
    pub minutes: Option<u16>,
}

impl Takeover {
    pub fn new() -> Self {
        Takeover::default()
    }

    fn file() -> PathBuf {
        let d = match env::var("RUST_TEST") {
            Ok(_) => env::current_dir().unwrap_or_default(),
            Err(_) => dirs::home_dir().unwrap_or_default(),
        };
        d.join(".trackrs-takeover")
    }

    pub fn set(&mut self, minutes: u16) -> Result {
        if minutes == 0 {
            log::warn!("won't take over less than 0 minutes");
            Ok(Takeover::default())
        } else {
            let file = Takeover::file();
            self.minutes = Some(minutes);

            log::debug!("takeover {} minutes next time", minutes);
            let w = OpenOptions::new()
                .create(true)
                .write(true)
                .append(false)
                .truncate(false)
                .open(file)?;
            serde_json::to_writer(w, &self)?;
            Ok(self.to_owned())
        }
    }

    pub fn get(&self) -> Result {
        let file = Takeover::file();
        if file.exists() {
            log::debug!("takeover was requested");
            let f = File::open(&file)?;
            let mut t: Takeover = serde_json::from_reader(f)?;
            if t.minutes.is_none() || t.minutes.as_ref().unwrap() <= &0 {
                t.minutes = None;
            }
            Ok(t)
        } else {
            log::debug!("no takeover requested");
            Ok(Takeover::default())
        }
    }
}

#[cfg(test)]
mod tests {

    use crate::test_utils::init;
    use crate::Takeover;
    use crate::TrackerError;
    use std::env::set_current_dir;
    use std::fs;
    use std::fs::File;
    use std::io::Write;

    mod get {

        use super::*;

        #[test]
        fn file_not_exists() -> Result<(), TrackerError> {
            init();
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let t = Takeover::new().get()?;
            assert!(t.minutes.is_none());
            Ok(())
        }

        #[test]
        fn file_contains_time() -> Result<(), TrackerError> {
            init();
            let file_content = "{\"minutes\":15}";
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let t = Takeover::new().get()?;

            assert!(t.minutes.is_some());
            assert_eq!(&15, t.minutes.as_ref().unwrap());

            Ok(())
        }

        #[test]
        fn file_contains_zero_time() -> Result<(), TrackerError> {
            init();
            let file_content = "{\"minutes\":0}";
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let t = Takeover::new().get()?;

            assert!(t.minutes.is_none());
            Ok(())
        }
    }

    mod set {

        use super::*;

        #[test]
        fn create_takeover_file() -> Result<(), TrackerError> {
            init();
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut b = Takeover::new();
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
            init();
            let temp_dir = tempfile::tempdir()?;
            set_current_dir(temp_dir.as_ref())?;
            let time_file = temp_dir.path().join(".trackrs-takeover");
            let mut b = Takeover::new();
            let t = b.set(0)?;

            assert!(!time_file.exists());
            assert!(t.minutes.is_none());
            Ok(())
        }
    }
}
