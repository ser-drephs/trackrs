use std::{ fs::{ self, File, OpenOptions }, io::BufReader, ops::Sub, path::PathBuf, str::FromStr };

use chrono::{ DateTime, Duration, Utc };

use crate::{ Entries, Entry, Settings, Status, Takeover, TrackerError, Upgrade };

pub type TimeDataResult = Result<TimeData, TrackerError>;
pub type TimeDataWriteResult = Result<(), TrackerError>;

#[derive(Default, Clone, Debug)]
pub struct TimeData {
    pub entries: Entries,
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
            let last_id = match self.entries.data.last() {
                Some(last) => last.id,
                None => 0,
            };
            let now = self.entries.data.last().unwrap().time;

            // calculate time for break assertion start
            let diff_b = e_break - a_break + Duration::minutes(1);
            let time_b = now - diff_b;

            // overwrite end entry
            let last_index = last_id - 1;
            self.entries.data[last_index as usize].status = Status::Break;
            self.entries.data[last_index as usize].time = time_b;

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
                self.entries.data[last_index as usize],
                entry_c
            );

            self.entries.data.append(&mut [entry_c, entry_e].to_vec());
        }
        Ok(self)
    }

    pub fn append(
        &mut self,
        status: Status,
        time: DateTime<Utc>,
        note: Option<String>
    ) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let last_id = match self.entries.data.last() {
            Some(last) => last.id,
            None => 0,
        };

        let entry = Entry::builder()
            .id(last_id)
            .status(status)
            .time(time.to_utc())
            .note(note)
            .build()?;
        log::debug!("append time data: {:?}", entry);
        self.entries.data.append(&mut [entry].to_vec());
        Ok(self)
    }

    /// Fill the remaining target worktime of the day with an absence (`sick` or `vacation`).
    ///
    /// The absence starts at the default start (`date` of this time data) for an empty day, at the
    /// end of an ended day (whose end is replaced), and otherwise at `now`, or at the last entry if
    /// the day is not today. It lasts until the day's worktime reaches the target with its break.
    pub fn absence(
        &mut self,
        note: &str,
        now: DateTime<Utc>,
        settings: &Settings
    ) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let day = self.date.unwrap();
        let mut data = self.entries.data.clone();
        let from = match data.last().map(|e| (e.status.to_owned(), e.time)) {
            None => day,
            Some((Status::End, time)) => {
                data.pop();
                time
            }
            Some((Status::Connect | Status::Break, time)) => {
                if now.date_naive() == day.date_naive() { now.max(time) } else { time }
            }
            Some((status, _)) => {
                return Err(TrackerError::TimeDataError {
                    message: format!("can't add {note} to a day ending with {status:?}"),
                });
            }
        };

        let start = data
            .iter()
            .find(|e| e.status == Status::Connect)
            .map_or(from, |e| e.time);
        let taken = data
            .iter()
            .enumerate()
            .filter(|(_, e)| e.status == Status::Break)
            .map(|(i, e)| data.get(i + 1).map_or(from, |n| n.time) - e.time)
            .fold(Duration::zero(), |a, b| a + b);
        let target = Duration::minutes((*settings.workperday.from(start)).into());
        let end = start + target + taken.max(settings.break_for(target));
        if end <= from {
            return Err(TrackerError::TimeDataError {
                message: format!("target worktime already reached, not adding {note}"),
            });
        }

        self.entries.data = data;
        self.append(Status::Connect, from, Some(note.to_owned()))?.append(Status::End, end, None)
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
            self.entries.data.append(&mut [t_entry].to_vec());
        }
        Ok(self)
    }

    pub fn takeover(&mut self, takeover: Duration) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let end = self.entries.data.iter().find(|&x| x.status == Status::End);
        if end.is_none() {
            log::warn!("End first to takeover time!");
        } else {
            let last_id = self.entries.data.last().unwrap().id;
            let old_time = Option::unwrap(end).time;
            self.entries.data[(last_id - 1) as usize].time = old_time.sub(takeover);

            let entry = Entry::builder()
                .id(last_id)
                .status(Status::Takeover)
                .time(old_time)
                .build()?;
            log::debug!("append takeover: {:?}", entry);
            self.entries.data.append(&mut [entry].to_vec());
        }
        Ok(self)
    }

    pub fn read_from_file(&mut self) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if self.file.exists() {
            let f = File::open(&self.file)?;
            self.entries = Upgrade::upgrade(BufReader::new(f))?;

            self.entries.data.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
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

    mod absence {
        use chrono::{ DateTime, Utc };

        use crate::{ BreakLimit, Entries, Settings, StatusDaily };

        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<Utc> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap()
        }

        fn settings() -> Settings {
            Settings {
                limits: vec![BreakLimit { start: 6 * 60, minutes: 30 }],
                ..Default::default()
            }
        }

        /// Monday 2022-02-07 with the given (status, hour, minute) entries.
        fn day(entries: &[(Status, u32, u32)]) -> TimeData {
            let data = entries
                .iter()
                .enumerate()
                .map(|(i, (status, h, m))| Entry {
                    id: (i + 1) as u8,
                    status: status.to_owned(),
                    time: at(*h, *m),
                    note: None,
                })
                .collect();
            TimeData {
                entries: Entries { data, version: 2 },
                build: true,
                date: Some(at(6, 0)),
                ..Default::default()
            }
        }

        fn summary(t: &TimeData) -> Vec<(Status, DateTime<Utc>, Option<String>)> {
            t.entries.data
                .iter()
                .map(|e| (e.status.to_owned(), e.time, e.note.to_owned()))
                .collect()
        }

        fn worktime(t: &TimeData) -> String {
            let status = StatusDaily::builder()
                .data(t.to_owned())
                .settings(settings())
                .build()
                .unwrap();
            status.worktime.to_string()
        }

        fn sick() -> Option<String> {
            Some("sick".to_owned())
        }

        #[test]
        fn should_fill_empty_day() -> Result<(), TrackerError> {
            let mut t = day(&[]);
            t.absence("sick", at(9, 0), &settings())?;

            assert_eq!(
                vec![(Status::Connect, at(6, 0), sick()), (Status::End, at(14, 30), None)],
                summary(&t)
            );
            assert_eq!("08:00", worktime(&t));
            Ok(())
        }

        #[test]
        fn should_extend_running_day_from_now() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0)]);
            t.absence("sick", at(10, 0), &settings())?;

            assert_eq!(
                vec![
                    (Status::Connect, at(8, 0), None),
                    (Status::Connect, at(10, 0), sick()),
                    (Status::End, at(16, 30), None)
                ],
                summary(&t)
            );
            assert_eq!("08:00", worktime(&t));
            Ok(())
        }

        #[test]
        fn should_extend_ended_day_from_its_end() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 10, 0)]);
            t.absence("sick", at(15, 0), &settings())?;

            assert_eq!(
                vec![
                    (Status::Connect, at(8, 0), None),
                    (Status::Connect, at(10, 0), sick()),
                    (Status::End, at(16, 30), None)
                ],
                summary(&t)
            );
            assert_eq!("08:00", worktime(&t));
            Ok(())
        }

        #[test]
        fn should_keep_longer_break_taken() -> Result<(), TrackerError> {
            let mut t = day(
                &[(Status::Connect, 8, 0), (Status::Break, 12, 0), (Status::Connect, 13, 0)]
            );
            t.absence("sick", at(14, 0), &settings())?;

            assert_eq!((Status::End, at(17, 0), None), summary(&t)[4]);
            assert_eq!("08:00", worktime(&t));
            Ok(())
        }

        #[test]
        fn should_end_running_break() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::Break, 12, 0)]);
            t.absence("sick", at(12, 15), &settings())?;

            assert_eq!(
                vec![
                    (Status::Connect, at(8, 0), None),
                    (Status::Break, at(12, 0), None),
                    (Status::Connect, at(12, 15), sick()),
                    (Status::End, at(16, 30), None)
                ],
                summary(&t)
            );
            assert_eq!("08:00", worktime(&t));
            Ok(())
        }

        #[test]
        fn should_extend_past_day_from_last_entry() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0)]);
            t.absence("sick", Utc.with_ymd_and_hms(2022, 2, 9, 9, 0, 0).unwrap(), &settings())?;

            assert_eq!((Status::Connect, at(8, 0), sick()), summary(&t)[1]);
            assert_eq!((Status::End, at(16, 30), None), summary(&t)[2]);
            Ok(())
        }

        #[test]
        fn should_not_change_day_with_reached_target() {
            let mut t = day(&[(Status::Connect, 6, 0), (Status::End, 14, 30)]);
            let before = summary(&t);

            assert!(t.absence("vacation", at(15, 0), &settings()).is_err());
            assert_eq!(before, summary(&t));
        }

        #[test]
        fn should_not_change_day_ending_with_takeover() {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 10, 0), (Status::Takeover, 10, 0)]);
            let before = summary(&t);

            assert!(t.absence("sick", at(11, 0), &settings()).is_err());
            assert_eq!(before, summary(&t));
        }
    }

    mod time_data {
        use std::{ io::Read, process::Command };

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
                    day.with_time(NaiveTime::from_hms_opt(2, 1, 0).unwrap()).unwrap(),
                    None
                )?
                .append(
                    Status::End,
                    day.with_time(NaiveTime::from_hms_opt(4, 1, 0).unwrap()).unwrap(),
                    None
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
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}],\"version\":1}";
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
            assert_eq!(2, time_data.entries.data.len());
            assert_eq!(2, time_data.entries.data.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_read_and_write_time_data() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}],\"version\":1}";
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
            assert_eq!(2, time_data.entries.data.len());
            assert_eq!(2, time_data.entries.data.last().unwrap().id);

            time_data.append(
                Status::End,
                day.with_time(NaiveTime::from_hms_opt(23, 3, 0).unwrap()).unwrap(),
                None
            )?;
            assert_eq!(3, time_data.entries.data.len());
            assert_eq!(3, time_data.entries.data.last().unwrap().id);

            time_data.write_to_file()?;
            assert!(fs::metadata(&time_file)?.len() > initial_size);
            Ok(())
        }

        #[test]
        fn should_update_and_write_time_data() -> Result<(), TrackerError> {
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
            assert_eq!(2, time_data.entries.data.len());
            assert_eq!(2, time_data.entries.data.last().unwrap().id);

            time_data.append(
                Status::End,
                day.with_time(NaiveTime::from_hms_opt(23, 3, 0).unwrap()).unwrap(),
                None
            )?;
            assert_eq!(3, time_data.entries.data.len());
            assert_eq!(3, time_data.entries.data.last().unwrap().id);

            time_data.write_to_file()?;
            assert!(fs::metadata(&time_file)?.len() > initial_size);

            let file_content_update =
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"},{\"id\":3,\"status\":\"End\",\"time\":\"2022-08-04T23:03:00Z\"}],\"version\":2}";
            let mut update_file = File::open(&time_file).unwrap();
            let mut data = vec![];
            update_file.read_to_end(&mut data)?;
            let repr = String::from_utf8(data).unwrap();

            assert_eq!(repr, file_content_update, "text compare");
            Ok(())
        }

        #[test]
        fn should_assert_break() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";
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
            assert_eq!(4, time_data.entries.data.len());
            assert_eq!(4, time_data.entries.data.last().unwrap().id);

            let r#break = match time_data.entries.data.iter().find(|x| x.status == Status::Break) {
                Some(c) => c.into(),
                None => None,
            };

            let connects = time_data.entries.data
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
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}],\"version\":1}";
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
            assert_eq!(4, time_data.entries.data.len());
            assert_eq!(4, time_data.entries.data.last().unwrap().id);

            // assert order of elements
            let fill_connect = time_data.entries.data[1].to_owned();
            let fill_break = time_data.entries.data[2].to_owned();
            let end = time_data.entries.data[3].to_owned();
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
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"Break\",\"time\":\"2022-08-04T00:30:53.523319900Z\"},{\"id\":3,\"status\":\"Connect\",\"time\":\"2022-08-04T02:15:53.523319900Z\"},{\"id\":4,\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}],\"version\":1}";
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
            assert_eq!(4, time_data.entries.data.len());
            assert_eq!(4, time_data.entries.data.last().unwrap().id);

            let r#break = match time_data.entries.data.iter().find(|x| x.status == Status::Break) {
                Some(c) => c.into(),
                None => None,
            };

            let connects = time_data.entries.data
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
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T04:00:53.523332900Z\"}],\"version\":1}";
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
            assert_eq!(2, time_data.entries.data.len());
            assert_eq!(2, time_data.entries.data.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_read_from_pretty_json() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "{\"data\":[\n    {\n        \"id\": 1,\n        \"status\": \"Connect\",\n        \"time\": \"2022-08-04T00:00:53.523319900Z\"\n    },\n    {\n        \"id\": 2,\n        \"status\": \"End\",\n        \"time\": \"2022-08-04T04:00:53.523332900Z\"\n    }\n], \"version\":1 \n}";
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
            assert_eq!(2, time_data.entries.data.len());
            assert_eq!(2, time_data.entries.data.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_read_from_pretty_json_carriage_return() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "{\r\n    \"data\": [\r\n        {\r\n            \"id\": 1,\r\n            \"status\": \"Connect\",\r\n            \"time\": \"2022-08-04T00:00:53.523319900Z\"\r\n        },\r\n        {\r\n            \"id\": 2,\r\n            \"status\": \"End\",\r\n            \"time\": \"2022-08-04T04:00:53.523332900Z\"\r\n        }\r\n    ],\r\n    \"version\": 1\r\n}";
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
            assert_eq!(2, time_data.entries.data.len());
            assert_eq!(2, time_data.entries.data.last().unwrap().id);
            Ok(())
        }

        #[test]
        fn should_create_takeover_entry() -> Result<(), TrackerError> {
            logger();
            let file_content =
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T04:00:53.523332900Z\"}],\"version\":1}";
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
            assert_eq!(3, time_data.entries.data.len());

            let end = time_data.entries.data[1].to_owned();
            assert_eq!(2, end.id);
            assert_eq!(Status::End, end.status);
            assert_eq!((3, 40, 53), (end.time.hour(), end.time.minute(), end.time.second()));

            let last = time_data.entries.data.last().unwrap();
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
                        day.with_time(NaiveTime::from_hms_opt(2, 16, 0).unwrap()).unwrap(),
                        None
                    )?
                    .append(
                        Status::End,
                        day.with_time(NaiveTime::from_hms_opt(5, 0, 0).unwrap()).unwrap(),
                        None
                    )?
                    .write_to_file()?;
                assert!(&time_file.exists(), "time file should exist");

                let takeover_time = time_data.entries.data[0].to_owned();
                let first_connect = time_data.entries.data[1].to_owned();
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
                        ),
                        None
                    )?
                    .append(
                        Status::End,
                        DateTime::from(
                            day.with_time(NaiveTime::from_hms_opt(2, 45, 0).unwrap()).unwrap()
                        ),
                        None
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
                        day.with_time(NaiveTime::from_hms_opt(2, 15, 0).unwrap()).unwrap(),
                        None
                    )?
                    .append(
                        Status::End,
                        day.with_time(NaiveTime::from_hms_opt(4, 14, 0).unwrap()).unwrap(),
                        None
                    )?
                    .write_to_file()?;
                assert!(&time_file.exists());

                let takeover_time = time_data.entries.data[0].to_owned();
                let first_connect = time_data.entries.data[1].to_owned();
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
