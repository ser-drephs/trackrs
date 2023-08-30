use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
};

use crate::{
    deprecated::Entry,
    dto::TrackerData,
    storage::{Folder, StorageError},
    Settings,
};

use super::Storage;

struct FileStorage {
    folder: Folder,
}

impl FileStorage {
    pub fn new(settings: &Settings) -> Self {
        FileStorage {
            folder: Folder::from(settings.folder.clone()),
        }
    }

    fn format_filename(&self, date: &chrono::NaiveDate) -> PathBuf {
        let df = date.format("%Y%m%d");
        self.folder.join(&format!("{}.json", df))
    }
}

impl Storage for FileStorage {
    fn read(&self, date: &chrono::NaiveDate) -> Result<crate::dto::TrackerData, StorageError> {
        let file = self.format_filename(date);
        if file.exists() {
            log::debug!("file at {:?} exists, continue reading.", file);
            let f = File::open(&file)?;
            let data: TrackerData = match serde_json::from_reader(&f) {
                Ok(res) => res,
                Err(_) => {
                    let entries: Vec<Entry> = serde_json::from_reader(f)?;
                    TrackerData::from(&entries)
                }
            };
            Ok(data)
        } else {
            Err(StorageError::FileNotFound {
                file: file.as_path().to_str().unwrap().to_string(),
            })
        }
    }

    fn read_all(&self) -> Result<Vec<TrackerData>, StorageError> {
        todo!()
        // implement logic to get all *.json files in that folder and pass them to read.
        // let folder = Folder::from(self.settings.folder);
        // let mut data_collection: Vec<TrackerData> = vec![];
        // for n in 0..10 {
        //     let date = NaiveDate::from_isoywd(2023, 23, chrono::Weekday::Fri);
        //     let data = self.read(date)?;
        //     data_collection.append(&mut [(data)].to_vec());
        // }
        // Ok(data_collection)
    }

    fn write(&self, data: &TrackerData) -> Result<(), StorageError> {
        fs::create_dir_all(&self.folder)?;
        let file = self.format_filename(&data.date);

        log::debug!("write data to file at {:?}", &file);
        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(&file)?;

        Ok(serde_json::to_writer(w, &data)?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{storage::{file_storage::FileStorage, Storage}, test_helper, Settings};

    #[test]
    fn should_read_old_file_format() {
        test_helper::setup();

        let temp_dir = tempfile::tempdir().unwrap();
        test_helper::write_test_file(&temp_dir, "20220804.json", "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}]").unwrap();

        let mut settings = Settings::new().unwrap();
        settings.folder = temp_dir.path().to_str().unwrap().to_string();

        let storage = FileStorage::new(&settings);
        let data = storage.read(&NaiveDate::from_ymd(2022, 8, 4)).unwrap();

        assert_eq!(2, data.items.len());
        assert_eq!(2, data.items.last().unwrap().id());
    }
}
