use std::{
    fs::{self, File, OpenOptions},
    path::PathBuf,
    str::FromStr,
};

use chrono::{Date, Duration, Local};

use crate::{Entry, Status, TrackerError};

pub type TimeDataResult = Result<TimeData, TrackerError>;
pub type TimeDataWriteResult = Result<(), TrackerError>;

trait TimeDataFile {
    fn read_from_file(&mut self) -> Result<&mut Self, TrackerError>;

    fn write_to_file(&self) -> Result<(), TrackerError>;
}

#[derive(Default, Clone, Debug)]
pub struct TimeData {
    pub entries: Vec<Entry>,
    pub(super) file: PathBuf,
    pub(super) build: bool,
    pub date: Option<Date<Local>>,
}

impl TimeData {
    pub fn builder() -> TimeDataBuilder {
        TimeDataBuilder {
            inner: TimeData::default(),
            folder: PathBuf::default(),
            has_file: false,
        }
    }

    pub fn assert_break(
        &mut self,
        e_break: Duration,
        a_break: Duration,
    ) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if e_break > a_break {
            let last_id = match self.entries.last() {
                Some(last) => last.id,
                None => 0,
            };
            let now = self.entries.last().unwrap().time;

            // calculate time for break assertion start
            let diff_b = (e_break - a_break) + Duration::minutes(1);
            let time_b = now - diff_b;

            // overwrite end entry
            let last_index = last_id - 1;
            self.entries[last_index as usize].status = Status::Break;
            self.entries[last_index as usize].time = time_b;

            // calculate time for connect entry afterwards
            let local_c = Duration::minutes(1);
            let time_c = now - local_c;
            let entry_c = Entry::builder()
                .id(last_id)
                .status(Status::Connect)
                .time(time_c)
                .build();

            let entry_e = Entry::builder()
                .id(last_id + 1)
                .status(Status::End)
                .time(now)
                .build();
            log::debug!(
                "fill break with {:?} and {:?}",
                self.entries[last_id as usize],
                entry_c
            );

            self.entries.append(&mut [entry_c, entry_e].to_vec());
        }
        Ok(self)
    }

    pub fn append(&mut self, status: Status) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let last_id = match self.entries.last() {
            Some(last) => last.id,
            None => 0,
        };

        let entry = Entry::builder().id(last_id).status(status).build();
        log::debug!("append time data: {:?}", entry);
        self.entries.append(&mut [entry].to_vec());
        Ok(self)
    }

    pub fn read_from_file(&mut self) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if self.file.exists() {
            let f = File::open(&self.file)?;
            self.entries = serde_json::from_reader(f)?;
            self.entries
                .sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        } else {
            log::info!("file not yet created: {:?}", &self.file);
        }

        Ok(self)
    }

    pub fn write_to_file(&self) -> Result<(), TrackerError> {
        self.assert_build()?;
        log::debug!("write data to time file at {:?}", &self.file);
        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(&self.file)?;
        Ok(serde_json::to_writer(w, &self.entries)?)
    }

    fn assert_build(&self) -> Result<(), TrackerError> {
        if !self.build {
            Err(TrackerError::TimeDataError {
                message: "time data not build".to_string(),
            })
        } else {
            Ok(())
        }
    }
}

pub struct TimeDataBuilder {
    inner: TimeData,
    folder: PathBuf,
    has_file: bool,
}

#[derive(Debug, Default, Clone)]
pub struct Folder {
    inner: PathBuf,
}

impl From<PathBuf> for Folder {
    fn from(path: PathBuf) -> Self {
        Folder { inner: path }
    }
}

impl From<&str> for Folder {
    fn from(str: &str) -> Self {
        Folder {
            inner: PathBuf::from_str(str).unwrap(),
        }
    }
}

impl From<String> for Folder {
    fn from(str: String) -> Self {
        Folder {
            inner: PathBuf::from_str(&str).unwrap(),
        }
    }
}

impl From<Folder> for PathBuf {
    fn from(val: Folder) -> Self {
        val.inner
    }
}

impl TimeDataBuilder {
    pub fn folder(&mut self, folder: Folder) -> &mut Self {
        log::debug!("set time data folder to: {:?}", &folder);
        fs::create_dir_all(&folder.inner).unwrap();
        self.folder = folder.into();
        self
    }

    pub fn today(&mut self) -> &mut Self {
        self.date(Local::today())
    }

    pub fn date(&mut self, date: Date<Local>) -> &mut Self {
        let df = date.format("%Y%m%d");
        let file = self.folder.join(format!("{}.json", df));
        log::debug!("set time data file to: {:?}", &file);
        log::info!("time data for {}", date.format("%Y-%m-%d"));
        self.inner.file = file;
        self.has_file = true;
        self.inner.date = Some(date);
        self
    }

    pub fn build(&mut self) -> Result<TimeData, TrackerError> {
        if !self.has_file {
            Err(TrackerError::TimeDataError {
                message: "time data file is not defined".to_string(),
            })
        } else {
            self.inner.build = true;
            log::debug!("build time data: {:?}", &self.inner);
            Ok(self.inner.clone())
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs::{self, File},
        io::Write,
        ops::Add,
    };

    use chrono::{Duration, Local, TimeZone, Timelike};

    use crate::{Entry, Status, TimeData, TrackerError};

    fn logger() {
        // std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod builder {

        use super::*;

        #[test]
        fn should_error_with_file_not_set() {
            logger();
            let time_data = TimeData::builder().build();
            assert!(time_data.is_err());
        }

        #[test]
        fn should_error_with_date_not_set() {
            logger();
            let time_data = TimeData::builder().folder("".into()).build();
            assert!(time_data.is_err());
        }
    }

    mod time_data {

        use super::*;

        #[test]
        fn should_write_time_data() -> Result<(), TrackerError> {
            logger();
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;
            time_data
                .read_from_file()?
                .append(Status::Connect)?
                .append(Status::End)?
                .write_to_file()?;
            assert!(&time_file.exists());
            assert!(fs::metadata(&time_file)?.len() > 0);
            Ok(())
        }

        #[test]
        fn should_read_time_data() -> Result<(), TrackerError> {
            logger();
            let file_content = "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;

            time_data.read_from_file()?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_read_and_write_time_data() -> Result<(), TrackerError> {
            logger();
            let file_content = "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;
            let initial_size = fs::metadata(&time_file)?.len();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;

            time_data.read_from_file()?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);

            time_data.append(Status::End)?;
            assert_eq!(3, time_data.entries.len());
            assert_eq!(3, time_data.entries.last().unwrap().id);

            time_data.write_to_file()?;
            assert!(fs::metadata(&time_file)?.len() > initial_size);
            Ok(())
        }

        #[test]
        fn should_assert_break() -> Result<(), TrackerError> {
            logger();
            let file_content = "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;

            time_data
                .read_from_file()?
                .assert_break(Duration::minutes(45), Duration::minutes(15))?;
            assert_eq!(4, time_data.entries.len());
            assert_eq!(4, time_data.entries.last().unwrap().id);

            let r#break = match time_data.entries.iter().find(|x| x.status == Status::Break) {
                Some(c) => c.into(),
                None => None,
            };

            let connects = time_data
                .entries
                .iter()
                .filter(|x| x.status == Status::Connect)
                .cloned()
                .collect::<Vec<Entry>>();
            let connect = connects.last();

            assert!(r#break.is_some());
            assert!(connect.is_some());

            assert_eq!(r#break.unwrap().id + 1, connect.unwrap().id);

            let duration_b = r#break.unwrap().time.num_seconds_from_midnight();
            let duration_c = connect.unwrap().time.num_seconds_from_midnight();
            assert_eq!(1800, duration_c - duration_b);
            Ok(())
        }

        #[test]
        fn should_assert_break_with_order() -> Result<(), TrackerError> {
            logger();
            let file_content = "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;

            time_data
                .read_from_file()?
                .assert_break(Duration::minutes(45), Duration::minutes(15))?
                .write_to_file()?;
            assert_eq!(4, time_data.entries.len());
            assert_eq!(4, time_data.entries.last().unwrap().id);

            // assert order of elements
            let fill_connect = time_data.entries[1].to_owned();
            let fill_break = time_data.entries[2].to_owned();
            let end = time_data.entries[3].to_owned();
            assert_eq!(
                (2, Status::Break),
                (fill_connect.id, fill_connect.status.to_owned()),
                "expected filler connect entry to be 2 and status Break but got {} {:?}",
                fill_connect.id,
                fill_connect.status
            );
            assert_eq!(
                (3, Status::Connect),
                (fill_break.id, fill_break.status.to_owned()),
                "expected filler break entry to be 3 and status Connect but got {} {:?}",
                fill_break.id,
                fill_break.status
            );
            assert_eq!(
                (4, Status::End),
                (end.id, end.status.to_owned()),
                "expected end entry to be 4 and status End but got {} {:?}",
                end.id,
                end.status
            );

            // assert file content
            let exp_break = "{\"id\":2,\"status\":\"Break\",\"time\":\"2022-08-04T07:29";

            let exp_connect = "{\"id\":3,\"status\":\"Connect\",\"time\":\"2022-08-04T07:59";

            let exp_end = "{\"id\":4,\"status\":\"End\",\"time\":\"2022-08-04T08:00";

            let act_content = fs::read_to_string(time_file)?;

            assert!(
                act_content.contains(exp_break),
                "expected content to contain Break with ID 2 and time 07:29, but got {}",
                act_content
            );
            assert!(
                act_content.contains(exp_connect),
                "expected content to contain Connect with ID 3 and time 07:59, but got {}",
                act_content
            );
            assert!(
                act_content.contains(exp_end),
                "expected content to contain End with ID 4 and time 08:00, but got {}",
                act_content
            );
            Ok(())
        }

        #[test]
        fn should_assert_long_break_do_nothing() -> Result<(), TrackerError> {
            logger();
            let file_content = "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"Break\",\"time\":\"2022-08-04T00:30:53.523319900Z\"},{\"id\":3,\"status\":\"Connect\",\"time\":\"2022-08-04T02:15:53.523319900Z\"},{\"id\":4,\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;

            time_data.read_from_file()?.assert_break(
                Duration::minutes(45),
                Duration::hours(1).add(Duration::minutes(15)),
            )?;
            assert_eq!(4, time_data.entries.len());
            assert_eq!(4, time_data.entries.last().unwrap().id);

            let r#break = match time_data.entries.iter().find(|x| x.status == Status::Break) {
                Some(c) => c.into(),
                None => None,
            };

            let connects = time_data
                .entries
                .iter()
                .filter(|x| x.status == Status::Connect)
                .cloned()
                .collect::<Vec<Entry>>();
            let connect = connects.last();

            assert!(r#break.is_some(), "break should be set");
            assert!(connect.is_some(), "connect should be set");

            let duration_b = r#break.unwrap().time.num_seconds_from_midnight();
            let duration_c = connect.unwrap().time.num_seconds_from_midnight();
            assert_eq!(
                6300,
                duration_c - duration_b,
                "total break duration should be 30 minutes"
            );
            Ok(())
        }

        #[test]
        fn should_assert_break_and_do_nothing() -> Result<(), TrackerError> {
            logger();
            let file_content = "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T04:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;

            time_data
                .read_from_file()?
                .assert_break(Duration::minutes(15), Duration::minutes(45))?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_read_from_pretty_json() -> Result<(), TrackerError> {
            logger();
            let file_content = "[\n    {\n        \"id\": 1,\n        \"status\": \"Connect\",\n        \"time\": \"2022-08-04T00:00:53.523319900Z\"\n    },\n    {\n        \"id\": 2,\n        \"status\": \"End\",\n        \"time\": \"2022-08-04T04:00:53.523332900Z\"\n    }\n]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Local.ymd(2022, 2, 2))
                .build()?;

            time_data
                .read_from_file()?
                .assert_break(Duration::minutes(15), Duration::minutes(45))?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);
            Ok(())
        }
    }
}
