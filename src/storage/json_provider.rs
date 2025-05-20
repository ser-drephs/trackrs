use core::time;
use std::{ fs::{ File, OpenOptions }, io::BufReader, path::PathBuf };

use chrono::Utc;

use crate::{models::Timesheet, Upgrade};

use super::{ StorageProvider, StorageProviderError };

pub struct JsonStorageProvider {
    file: PathBuf,
}

impl JsonStorageProvider {
    pub fn new(mut file: PathBuf) -> Self {
        if !file.ends_with(".json") {
            file.set_extension("json");
        }
        return JsonStorageProvider { file };
    }

    pub fn new_today(folder: PathBuf) -> Result<Self, StorageProviderError> {
        let now = Utc::now();
        let date_str = format!("{}", now.format("%Y-%m-%d"));
        let file = folder.join(date_str);
        Ok(JsonStorageProvider::new(file))
    }
}

impl StorageProvider for JsonStorageProvider {
    fn read(&self) -> Result<Timesheet, StorageProviderError> {
        log::debug!("read timesheet using json provider");
        let mut timesheet = Timesheet::new();

        if self.file.exists() {
            log::debug!("file found appending data: {:?}", &self.file);
            let f = File::open(&self.file)?;

            match Upgrade::upgrade(BufReader::new(f))? {
                Some(res) => {
                    timesheet = res;
                }
                None => {
                    let f = File::open(&self.file)?;
                    timesheet = serde_json::from_reader(BufReader::new(f))?;
                }
            }
            timesheet.sort();
        } else {
            log::info!("file not yet created: {:?}", &self.file);
            // TODO: invoke takeover- here? or use some prepending logic?
        }
        Ok(timesheet)
    }

    fn write(&self, entries: &Timesheet) -> Result<(), StorageProviderError> {
        log::debug!("write timesheet using json provider");
        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(&self.file)?;
        log::debug!("write to file: {:?}", &self.file);
        Ok(serde_json::to_writer(w, &entries)?)
    }
}

#[cfg(test)]
mod tests {
    use super::JsonStorageProvider;
    use std::path::PathBuf;

    #[test]
    fn should_generate_file_name_for_today() {
        let provider = JsonStorageProvider::new_today("".into()).unwrap();
        assert!(provider.file.as_os_str().to_str().unwrap().ends_with("json"))
    }

    #[test]
    fn should_append_file_extension() {
        let path = PathBuf::from("sample");
        let provider = JsonStorageProvider::new(path);
        assert_eq!("sample.json", provider.file.as_os_str())
    }

    #[test]
    fn should_not_append_file_extension() {
        let path = PathBuf::from("sample2.json");
        let provider = JsonStorageProvider::new(path);
        assert_eq!("sample2.json", provider.file.as_os_str())
    }

    // TODO: add integration tests for file write and read with test context and tempdir
}
