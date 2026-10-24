use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use crate::TrackerError;

/// Minutes taken over from one day to the next start, stored in the time data folder.
pub struct Takeover {
    file: PathBuf,
}

#[derive(Deserialize, Serialize)]
struct Content {
    minutes: Option<u16>,
}

impl Takeover {
    pub fn new(folder: &Path) -> Self {
        Takeover {
            file: folder.join(".trackrs-takeover"),
        }
    }

    /// Pending minutes, if any.
    pub fn get(&self) -> Result<Option<u16>, TrackerError> {
        if !self.file.exists() {
            log::debug!("no takeover requested");
            return Ok(None);
        }
        let c: Content = serde_json::from_reader(File::open(&self.file)?)?;
        Ok(c.minutes.filter(|m| *m > 0))
    }

    pub fn set(&self, minutes: u16) -> Result<(), TrackerError> {
        log::debug!("takeover {} minutes next time", minutes);
        let c = Content {
            minutes: Some(minutes),
        };
        Ok(fs::write(&self.file, serde_json::to_string(&c)?)?)
    }

    pub fn remove(&self) -> Result<(), TrackerError> {
        if self.file.exists() {
            fs::remove_file(&self.file)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::{Takeover, TrackerError};

    #[test]
    fn should_get_nothing_without_file() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        assert_eq!(None, Takeover::new(temp_dir.path()).get()?);
        Ok(())
    }

    #[test]
    fn should_set_get_and_remove() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        let t = Takeover::new(temp_dir.path());
        t.set(25)?;

        let file = temp_dir.path().join(".trackrs-takeover");
        assert_eq!("{\"minutes\":25}", fs::read_to_string(&file)?);
        assert_eq!(Some(25), t.get()?);

        t.remove()?;
        assert!(!file.exists());
        assert_eq!(None, t.get()?);
        Ok(())
    }

    #[test]
    fn should_ignore_zero_minutes() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        fs::write(temp_dir.path().join(".trackrs-takeover"), "{\"minutes\":0}")?;
        assert_eq!(None, Takeover::new(temp_dir.path()).get()?);
        Ok(())
    }
}
