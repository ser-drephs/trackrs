use std::{ fs::{ self, File, OpenOptions }, ops::Sub, path::PathBuf, str::FromStr };

use chrono::{ DateTime, Duration, Utc };

use crate::{ Entry, Status, Takeover, TrackerError };

pub type TimeDataResult = Result<TimeData, TrackerError>;
pub type TimeDataWriteResult = Result<(), TrackerError>;

#[derive(Default, Clone, Debug)]
pub struct TimeData {
    pub entries: Vec<Entry>,
    pub(super) file: PathBuf,
    pub(super) build: bool,
    pub date: Option<DateTime<Utc>>,
    pub takeover: Option<Takeover>,
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
        a_break: Duration
    ) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if e_break > a_break {
            let last_id = match self.entries.last() {
                Some(last) => last.id,
                None => 0,
            };
            let now = self.entries.last().unwrap().time;

            // calculate time for break assertion start
            let diff_b = e_break - a_break + Duration::minutes(1);
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
                .build()?;

            let entry_e = Entry::builder()
                .id(last_id + 1)
                .status(Status::End)
                .time(now)
                .build()?;
            log::debug!(
                "fill break with {:?} and {:?}",
                self.entries[last_index as usize],
                entry_c
            );

            self.entries.append(&mut [entry_c, entry_e].to_vec());
        }
        Ok(self)
    }

    pub fn append(
        &mut self,
        status: Status,
        time: DateTime<Utc>
    ) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let last_id = match self.entries.last() {
            Some(last) => last.id,
            None => 0,
        };

        let entry = Entry::builder().id(last_id).status(status).time(time.to_utc()).build()?;
        log::debug!("append time data: {:?}", entry);
        self.entries.append(&mut [entry].to_vec());
        Ok(self)
    }

    pub fn assert_takeover(&mut self, time: DateTime<Utc>) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if self.takeover.is_some() {
            let m = self.takeover.as_ref().unwrap();
            let time = time.sub(Duration::minutes(m.minutes.unwrap().try_into()?));
            let t_entry = Entry::builder()
                .id(0)
                .status(Status::Connect)
                .time(time.to_utc())
                .build()?;
            self.entries.append(&mut [t_entry].to_vec());
        }
        Ok(self)
    }

    pub fn takeover(&mut self, takeover: Duration) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let end = self.entries.iter().find(|&x| x.status == Status::End);
        if end.is_none() {
            log::warn!("End first to takeover time!");
        } else {
            let last_id = self.entries.last().unwrap().id;
            let old_time = Option::unwrap(end).time;
            self.entries[(last_id - 1) as usize].time = old_time.sub(takeover);

            let entry = Entry::builder()
                .id(last_id)
                .status(Status::Takeover)
                .time(old_time)
                .build()?;
            log::debug!("append takeover: {:?}", entry);
            self.entries.append(&mut [entry].to_vec());
        }
        Ok(self)
    }

    pub fn read_from_file(&mut self) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if self.file.exists() {
            let f = File::open(&self.file)?;
            self.entries = serde_json::from_reader(f)?;
            self.entries.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
        } else {
            log::info!("file not yet created: {:?}", &self.file);
            // invoke takeover
            let mut b = Takeover::builder();
            let t = b.file().get()?;
            if t.minutes.is_some() {
                self.takeover = Some(t);
            }
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
        self.date(Utc::now())
    }

    pub fn date(&mut self, date: DateTime<Utc>) -> &mut Self {
        let df = date.format("%Y%m%d");
        let file = self.folder.join(format!("{}.json", df));
        log::debug!("set time data file to: {:?}", &file);
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
    use std::{ env, fs::{ self, File }, io::Write, ops::Add };

    use chrono::{ Duration, TimeZone, Timelike };

    use crate::{ Entry, Status, TimeData, TrackerError };

    use serial_test::serial;

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
        use std::process::Command;

        use chrono::{ NaiveTime, Utc };

        use super::*;

        fn wait() {
            let mut child = Command::new("sleep").arg("1").spawn().unwrap();
            let _result = child.wait().unwrap();
        }

        #[test]
        fn should_write_time_data() -> Result<(), TrackerError> {
            logger();
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let day = Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap();
            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(day)
                .build()?;
            time_data
                .read_from_file()?
                .append(
                    Status::Connect,
                    day.with_time(NaiveTime::from_hms_opt(2, 1, 0).unwrap()).unwrap()
                )?
                .append(
                    Status::End,
                    day.with_time(NaiveTime::from_hms_opt(4, 1, 0).unwrap()).unwrap()
                )?
                .write_to_file()?;

            wait();

            assert!(&time_file.exists());
            assert!(fs::metadata(&time_file)?.len() > 0);
            Ok(())
        }

        #[test]
        fn should_read_time_data() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220804.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Utc.with_ymd_and_hms(2022, 8, 4, 0, 0, 0).unwrap())
                .build()?;

            time_data.read_from_file()?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_read_and_write_time_data() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220804.json");
            let day = Utc.with_ymd_and_hms(2022, 8, 4, 0, 0, 0).unwrap();
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let initial_size = fs::metadata(&time_file)?.len();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(day)
                .build()?;

            time_data.read_from_file()?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);

            time_data.append(
                Status::End,
                day.with_time(NaiveTime::from_hms_opt(23, 3, 0).unwrap()).unwrap()
            )?;
            assert_eq!(3, time_data.entries.len());
            assert_eq!(3, time_data.entries.last().unwrap().id);

            time_data.write_to_file()?;
            assert!(fs::metadata(&time_file)?.len() > initial_size);
            Ok(())
        }

        #[test]
        fn should_assert_break() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap())
                .build()?;

            time_data.read_from_file()?.assert_break(Duration::minutes(45), Duration::minutes(15))?;
            assert_eq!(4, time_data.entries.len());
            assert_eq!(4, time_data.entries.last().unwrap().id);

            let r#break = match time_data.entries.iter().find(|x| x.status == Status::Break) {
                Some(c) => c.into(),
                None => None,
            };

            let connects = time_data.entries
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
            let file_content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap())
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
            let file_content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"Break\",\"time\":\"2022-08-04T00:30:53.523319900Z\"},{\"id\":3,\"status\":\"Connect\",\"time\":\"2022-08-04T02:15:53.523319900Z\"},{\"id\":4,\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap())
                .build()?;

            time_data
                .read_from_file()?
                .assert_break(
                    Duration::minutes(45),
                    Duration::hours(1).add(Duration::minutes(15))
                )?;
            assert_eq!(4, time_data.entries.len());
            assert_eq!(4, time_data.entries.last().unwrap().id);

            let r#break = match time_data.entries.iter().find(|x| x.status == Status::Break) {
                Some(c) => c.into(),
                None => None,
            };

            let connects = time_data.entries
                .iter()
                .filter(|x| x.status == Status::Connect)
                .cloned()
                .collect::<Vec<Entry>>();
            let connect = connects.last();

            assert!(r#break.is_some(), "break should be set");
            assert!(connect.is_some(), "connect should be set");

            let duration_b = r#break.unwrap().time.num_seconds_from_midnight();
            let duration_c = connect.unwrap().time.num_seconds_from_midnight();
            assert_eq!(6300, duration_c - duration_b, "total break duration should be 30 minutes");
            Ok(())
        }

        #[test]
        fn should_assert_break_and_do_nothing() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T04:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap())
                .build()?;

            time_data.read_from_file()?.assert_break(Duration::minutes(15), Duration::minutes(45))?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_read_from_pretty_json() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "[\n    {\n        \"id\": 1,\n        \"status\": \"Connect\",\n        \"time\": \"2022-08-04T00:00:53.523319900Z\"\n    },\n    {\n        \"id\": 2,\n        \"status\": \"End\",\n        \"time\": \"2022-08-04T04:00:53.523332900Z\"\n    }\n]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220202.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap())
                .build()?;

            time_data.read_from_file()?.assert_break(Duration::minutes(15), Duration::minutes(45))?;
            assert_eq!(2, time_data.entries.len());
            assert_eq!(2, time_data.entries.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_create_takeover_entry() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T04:00:53.523332900Z\"}]";
            let temp_dir = tempfile::tempdir()?;
            let time_file = temp_dir.path().join("20220804.json");
            let mut file = File::create(&time_file)?;
            file.write_all(file_content.as_bytes())?;

            wait();

            let mut time_data = TimeData::builder()
                .folder(temp_dir.into_path().into())
                .date(Utc.with_ymd_and_hms(2022, 8, 4, 0, 0, 0).unwrap())
                .build()?;

            time_data.read_from_file()?.takeover(Duration::minutes(20))?;
            assert_eq!(3, time_data.entries.len());

            let end = time_data.entries[1].to_owned();
            assert_eq!(2, end.id);
            assert_eq!(Status::End, end.status);
            assert_eq!((3, 40, 53), (end.time.hour(), end.time.minute(), end.time.second()));

            let last = time_data.entries.last().unwrap();
            assert_eq!(3, last.id);
            assert_eq!(Status::Takeover, last.status);
            assert_eq!((4, 0, 53), (last.time.hour(), last.time.minute(), last.time.second()));
            Ok(())
        }

        mod takeover {
            use chrono::{ DateTime, Utc };

            use super::*;

            struct TakeoverContext {
                temp_dir: tempfile::TempDir,
            }

            impl test_context::TestContext for TakeoverContext {
                fn setup() -> TakeoverContext {
                    logger();
                    env::set_var("RUST_TEST", "true");
                    let temp_dir = tempfile::tempdir().unwrap();
                    env::set_current_dir(&temp_dir).unwrap();
                    TakeoverContext { temp_dir }
                }

                fn teardown(self) {
                    self.temp_dir.close().unwrap();
                }
            }

            #[test_context::test_context(TakeoverContext)]
            #[test]
            #[serial]
            fn should_takeover_time(ctx: &mut TakeoverContext) -> Result<(), TrackerError> {
                let time_file = ctx.temp_dir.path().join("20220202.json");
                let day = Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap();

                let file_content = "{\"minutes\":15}";
                let takeover_file = ctx.temp_dir.path().join(".trackrs-takeover");
                let mut file = File::create(&takeover_file)?;
                file.write_all(file_content.as_bytes())?;

                wait();

                let mut time_data = TimeData::builder()
                    .folder(ctx.temp_dir.as_ref().to_owned().into())
                    .date(day)
                    .build()?;
                time_data
                    .read_from_file()?
                    .assert_takeover(
                        day.with_time(NaiveTime::from_hms_opt(2, 16, 0).unwrap()).unwrap()
                    )?
                    .append(
                        Status::Connect,
                        day.with_time(NaiveTime::from_hms_opt(2, 16, 0).unwrap()).unwrap()
                    )?
                    .append(
                        Status::End,
                        day.with_time(NaiveTime::from_hms_opt(5, 0, 0).unwrap()).unwrap()
                    )?
                    .write_to_file()?;
                assert!(&time_file.exists(), "time file should exist");

                let takeover_time = time_data.entries[0].to_owned();
                let first_connect = time_data.entries[1].to_owned();
                let diff = first_connect.time - takeover_time.time;

                assert_eq!(
                    Duration::minutes(15).num_seconds(),
                    diff.num_seconds(),
                    "expected 15 minutes diff, but got {}",
                    diff
                );
                assert_eq!(
                    day.with_time(NaiveTime::from_hms_opt(2, 1, 0).unwrap()).unwrap(),
                    takeover_time.time,
                    "takeover time doesnt match"
                );
                Ok(())
            }

            #[test_context::test_context(TakeoverContext)]
            #[test]
            #[serial]
            fn should_takeover_time_check_file(
                ctx: &mut TakeoverContext
            ) -> Result<(), TrackerError> {
                let time_file = ctx.temp_dir.path().join("20220202.json");
                let day = Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap();

                let file_content = "{\"minutes\":15}";
                let takeover_file = ctx.temp_dir.path().join(".trackrs-takeover");
                let mut file = File::create(&takeover_file)?;
                file.write_all(file_content.as_bytes())?;

                wait();

                let mut time_data = TimeData::builder()
                    .folder(ctx.temp_dir.as_ref().to_owned().into())
                    .date(DateTime::from(day))
                    .build()?;
                time_data
                    .read_from_file()?
                    .assert_takeover(
                        DateTime::from(
                            day.with_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap()).unwrap()
                        )
                    )?
                    .append(
                        Status::Connect,
                        DateTime::from(
                            day.with_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap()).unwrap()
                        )
                    )?
                    .append(
                        Status::End,
                        DateTime::from(
                            day.with_time(NaiveTime::from_hms_opt(2, 45, 0).unwrap()).unwrap()
                        )
                    )?
                    .write_to_file()?;
                assert!(&time_file.exists());

                // assert file content
                let exp_takeover = "{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T02:00";
                let exp_connect = "{\"id\":2,\"status\":\"Connect\",\"time\":\"2022-02-02T02:15";

                let act_content = fs::read_to_string(time_file)?;

                assert!(
                    act_content.contains(exp_takeover),
                    "expected content to contain Takeover Connect with ID 1, but got {}",
                    act_content
                );
                assert!(
                    act_content.contains(exp_connect),
                    "expected content to contain Connect with ID 2, but got {}",
                    act_content
                );

                Ok(())
            }

            #[test_context::test_context(TakeoverContext)]
            #[test]
            #[serial]
            fn should_takeover_time_over_an_hour(
                ctx: &mut TakeoverContext
            ) -> Result<(), TrackerError> {
                let time_file = ctx.temp_dir.path().join("20220202.json");
                let day = Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap();

                let file_content = "{\"minutes\":95}";
                let takeover_file = ctx.temp_dir.path().join(".trackrs-takeover");
                let mut file = File::create(&takeover_file)?;
                file.write_all(file_content.as_bytes())?;

                wait();

                let mut time_data = TimeData::builder()
                    .folder(ctx.temp_dir.as_ref().to_owned().into())
                    .date(day)
                    .build()?;
                time_data
                    .read_from_file()?
                    .assert_takeover(
                        day.with_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap()).unwrap()
                    )?
                    .append(
                        Status::Connect,
                        day.with_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap()).unwrap()
                    )?
                    .append(
                        Status::End,
                        day.with_time(NaiveTime::from_hms_opt(4, 14, 0).unwrap()).unwrap()
                    )?
                    .write_to_file()?;
                assert!(&time_file.exists());

                let takeover_time = time_data.entries[0].to_owned();
                let first_connect = time_data.entries[1].to_owned();
                let diff = first_connect.time - takeover_time.time;

                assert_eq!(
                    Duration::minutes(95).num_seconds(),
                    diff.num_seconds(),
                    "expected 1:35 minutes diff, but got {}",
                    diff
                );
                Ok(())
            }
        }
    }
}
