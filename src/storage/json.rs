use std::{ fs::{ self, File, OpenOptions }, io::BufReader, path::PathBuf };

use chrono::NaiveDate;

use crate::{ StorageProvider, Timesheet, TrackerError, Upgrade };

/// Stores the timesheet of each day as `YYYYMMDD.json` in a folder.
#[derive(Clone, Debug)]
pub struct JsonStorage {
    folder: PathBuf,
}

impl JsonStorage {
    pub fn new(folder: impl Into<PathBuf>) -> Self {
        Self { folder: folder.into() }
    }

    fn file(&self, date: NaiveDate) -> PathBuf {
        self.folder.join(format!("{}.json", date.format("%Y%m%d")))
    }
}

impl StorageProvider for JsonStorage {
    fn load(&self, date: NaiveDate) -> Result<Timesheet, TrackerError> {
        let file = self.file(date);
        if !file.exists() {
            log::info!("file not yet created: {:?}", file);
            return Ok(Timesheet::default());
        }
        let mut timesheet = Upgrade::upgrade(BufReader::new(File::open(file)?))?;
        timesheet.data.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
        Ok(timesheet)
    }

    fn save(&self, date: NaiveDate, timesheet: &Timesheet) -> Result<(), TrackerError> {
        let file = self.file(date);
        log::debug!("write data to time file at {:?}", file);
        fs::create_dir_all(&self.folder)?;
        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            // The file may be longer than the new content, e.g. after being reformatted by hand.
            .truncate(true)
            .open(file)?;
        Ok(serde_json::to_writer(w, timesheet)?)
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ FixedOffset, Local, NaiveDate, TimeZone, Utc };

    use super::*;
    use crate::Status;

    fn day() -> NaiveDate {
        NaiveDate::from_ymd_opt(2022, 8, 4).unwrap()
    }

    /// A storage in a new folder, with `content` as the file of `day()` if given.
    fn storage(content: Option<&str>) -> (tempfile::TempDir, JsonStorage, PathBuf) {
        let dir = tempfile::tempdir().unwrap();
        let storage = JsonStorage::new(dir.path());
        let file = storage.file(day());
        if let Some(content) = content {
            fs::write(&file, content).unwrap();
        }
        (dir, storage, file)
    }

    const V1: &str =
        "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}],\"version\":1}";

    #[test]
    fn should_load_empty_without_file() -> Result<(), TrackerError> {
        let (_dir, storage, _) = storage(None);

        assert!(storage.load(day())?.data.is_empty());
        Ok(())
    }

    #[test]
    fn should_load_stored_file() -> Result<(), TrackerError> {
        let (_dir, storage, _) = storage(Some(V1));

        assert_eq!(2, storage.load(day())?.data.len());
        Ok(())
    }

    #[test]
    fn should_load_sorted_by_time() -> Result<(), TrackerError> {
        let (_dir, storage, _) = storage(
            Some(
                "{\"data\":[{\"status\":\"End\",\"time\":\"2022-08-04T16:00:00Z\"},{\"status\":\"Connect\",\"time\":\"2022-08-04T08:00:00Z\"}],\"version\":2}"
            )
        );

        let statuses: Vec<_> = storage.load(day())?.data.into_iter().map(|e| e.status).collect();
        assert_eq!(vec![Status::Connect, Status::End], statuses);
        Ok(())
    }

    #[test]
    fn should_load_pretty_json() -> Result<(), TrackerError> {
        let (_dir, storage, _) = storage(
            Some(
                "{\"data\":[\n    {\n        \"id\": 1,\n        \"status\": \"Connect\",\n        \"time\": \"2022-08-04T00:00:53.523319900Z\"\n    },\n    {\n        \"id\": 2,\n        \"status\": \"End\",\n        \"time\": \"2022-08-04T04:00:53.523332900Z\"\n    }\n], \"version\":1 \n}"
            )
        );

        assert_eq!(2, storage.load(day())?.data.len());
        Ok(())
    }

    #[test]
    fn should_load_pretty_json_carriage_return() -> Result<(), TrackerError> {
        let (_dir, storage, _) = storage(
            Some(
                "{\r\n    \"data\": [\r\n        {\r\n            \"id\": 1,\r\n            \"status\": \"Connect\",\r\n            \"time\": \"2022-08-04T00:00:53.523319900Z\"\r\n        },\r\n        {\r\n            \"id\": 2,\r\n            \"status\": \"End\",\r\n            \"time\": \"2022-08-04T04:00:53.523332900Z\"\r\n        }\r\n    ],\r\n    \"version\": 1\r\n}"
            )
        );

        assert_eq!(2, storage.load(day())?.data.len());
        Ok(())
    }

    #[test]
    fn should_save_into_new_folder() -> Result<(), TrackerError> {
        let dir = tempfile::tempdir()?;
        let storage = JsonStorage::new(dir.path().join("new"));
        let mut timesheet = Timesheet::default();
        timesheet.append(Status::Connect, Utc.with_ymd_and_hms(2022, 8, 4, 8, 0, 0).unwrap().fixed_offset(), None)?;

        storage.save(day(), &timesheet)?;

        assert!(fs::metadata(storage.file(day()))?.len() > 0);
        assert_eq!(1, storage.load(day())?.data.len());
        Ok(())
    }

    #[test]
    fn should_update_stored_file() -> Result<(), TrackerError> {
        let (_dir, storage, file) = storage(
            Some(
                "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900+02:00\"},{\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900+02:00\"}],\"version\":4}"
            )
        );
        let initial_size = fs::metadata(&file)?.len();

        let mut timesheet = storage.load(day())?;
        let time = FixedOffset::east_opt(2 * 3600)
            .unwrap()
            .with_ymd_and_hms(2022, 8, 4, 23, 3, 0)
            .unwrap();
        timesheet.append(Status::End, time, None)?;
        storage.save(day(), &timesheet)?;

        assert!(fs::metadata(&file)?.len() > initial_size);
        assert_eq!(
            "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900+02:00\"},{\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900+02:00\"},{\"status\":\"End\",\"time\":\"2022-08-04T23:03:00+02:00\"}],\"version\":4}",
            fs::read_to_string(&file)?
        );
        Ok(())
    }

    #[test]
    fn should_keep_the_offsets_of_the_entries() -> Result<(), TrackerError> {
        // A day of travelling, each entry keeps the offset it was written with.
        let content =
            "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-08-04T08:00:00+02:00\"},{\"status\":\"Break\",\"time\":\"2022-08-04T14:00:00-04:00\"}],\"version\":4}";
        let (_dir, storage, file) = storage(Some(content));

        storage.save(day(), &storage.load(day())?)?;

        assert_eq!(content, fs::read_to_string(&file)?);
        Ok(())
    }

    #[test]
    fn should_update_old_files_to_latest_version_on_save() -> Result<(), TrackerError> {
        let versionless =
            "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T08:00:00Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T16:00:00Z\"}]";
        let v1 =
            "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T08:00:00Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T16:00:00Z\"}],\"version\":1}";
        let v2 =
            "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T08:00:00Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T16:00:00Z\"}],\"version\":2}";
        let v3 =
            "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-08-04T08:00:00Z\"},{\"status\":\"End\",\"time\":\"2022-08-04T16:00:00Z\"}],\"version\":3}";
        let instants = [
            Utc.with_ymd_and_hms(2022, 8, 4, 8, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2022, 8, 4, 16, 0, 0).unwrap(),
        ];

        for old in [versionless, v1, v2, v3] {
            let (_dir, storage, file) = storage(Some(old));

            storage.save(day(), &storage.load(day())?)?;

            let content = fs::read_to_string(&file)?;
            assert!(!content.contains("\"id\""), "{content}");
            assert!(content.ends_with("\"version\":4}"), "{content}");
            // The same instants, written in the local offset of the machine at that moment.
            let saved = storage.load(day())?;
            for (entry, instant) in saved.data.iter().zip(instants) {
                assert_eq!(instant, entry.time, "{content}");
                let local = Local.offset_from_utc_datetime(&instant.naive_utc());
                assert_eq!(local, *entry.time.offset(), "{content}");
            }
        }
        Ok(())
    }

    #[test]
    fn should_replace_longer_file() -> Result<(), TrackerError> {
        // Pretty printed, as after saving it in an editor, longer than written compactly.
        let (_dir, storage, file) = storage(
            Some(
                "{\n    \"data\": [\n        {\n            \"status\": \"Connect\",\n            \"time\": \"2022-02-07T07:00:00+01:00\"\n        }\n    ],\n    \"version\": 4\n}\n"
            )
        );
        let before = fs::metadata(&file)?.len();

        storage.save(day(), &storage.load(day())?)?;

        let content = fs::read_to_string(&file)?;
        assert!((content.len() as u64) < before, "{content}");
        assert_eq!(
            "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-02-07T07:00:00+01:00\"}],\"version\":4}",
            content
        );
        assert_eq!(1, storage.load(day())?.data.len());
        Ok(())
    }
}
