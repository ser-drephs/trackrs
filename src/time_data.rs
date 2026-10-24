use std::{ fs::{ self, File, OpenOptions }, io::BufReader, path::PathBuf, str::FromStr };

use chrono::{ DateTime, Duration, Utc };

use crate::{ Entries, Entry, Settings, Status, TrackerError, Upgrade };

pub type TimeDataResult = Result<TimeData, TrackerError>;
pub type TimeDataWriteResult = Result<(), TrackerError>;


#[derive(Default, Clone, Debug)]
pub struct TimeData {
    pub entries: Entries,
    pub(super) file: PathBuf,
    pub(super) build: bool,
    pub date: Option<DateTime<Utc>>,
}

impl TimeData {
    pub fn builder() -> TimeDataBuilder {
        TimeDataBuilder {
            inner: TimeData::default(),
            folder: PathBuf::default(),
            has_file: false,
        }
    }

    /// Fill the break missing to the expected break `e_break`, as a break with a note:
    /// - after the first break and connect pair, so that break looks extended,
    /// - or one second after the start, if no break was taken yet. Starts in front of it, like a
    ///   takeover or an addition, are skipped, so the filled break starts after the real start.
    ///
    /// The filled break is skipped if it would overlap the following entry.
    pub fn assert_break(
        &mut self,
        e_break: Duration,
        a_break: Duration
    ) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if e_break <= a_break {
            return Ok(self);
        }
        let data = &self.entries.data;
        let after_break = data
            .windows(2)
            .position(|w| w[0].status == Status::Break && w[1].status == Status::Connect)
            .map(|i| i + 1);
        let anchor = after_break.or_else(|| {
            let first = data.iter().position(|e| e.status == Status::Connect)?;
            let starts = data[first..].iter().take_while(|e| e.status == Status::Connect).count();
            Some(first + starts - 1)
        });
        let Some(anchor) = anchor else {
            return Ok(self);
        };

        let start = data[anchor].time + Duration::seconds(1);
        let end = start + (e_break - a_break);
        if let Some(next) = data.get(anchor + 1).filter(|n| n.time <= end) {
            eprintln!(
                "not filling the mandatory remaining break of {} minutes, it would overlap the {:?} entry",
                (e_break - a_break).num_minutes(),
                next.status
            );
            return Ok(self);
        }

        let r#break = Entry::builder()
            .status(Status::Break)
            .time(start)
            .note(Some("filled mandatory remaining break".to_owned()))
            .build()?;
        let connect = Entry::builder().status(Status::Connect).time(end).build()?;
        log::debug!("fill break with {:?} and {:?}", r#break, connect);
        self.entries.data.splice(anchor + 1..anchor + 1, [r#break, connect]);
        self.renumber();
        Ok(self)
    }

    /// ids are positions, so keep them in order after inserting entries.
    fn renumber(&mut self) {
        for (i, e) in self.entries.data.iter_mut().enumerate() {
            e.id = (i + 1) as u8;
        }
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

    /// Prepend a connect entry `minutes` before the first connect of the day.
    pub fn prepend(&mut self, minutes: u16, note: &str) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let first = self.entries.data
            .iter()
            .find(|e| e.status == Status::Connect)
            .ok_or_else(|| TrackerError::TimeDataError {
                message: "no start found for the day, start first".to_owned(),
            })?;
        let time = first.time - Duration::minutes(minutes.into());
        if time.date_naive() != first.time.date_naive() {
            return Err(TrackerError::TimeDataError {
                message: format!("adding {minutes} minutes would start before the day"),
            });
        }

        let entry = Entry::builder()
            .status(Status::Connect)
            .time(time)
            .note(Some(note.to_owned()))
            .build()?;
        log::debug!("prepend time data: {:?}", entry);
        self.entries.data.insert(0, entry);
        self.renumber();
        Ok(self)
    }

    /// Take over `minutes` of an ended day to the next start: move its end back by `minutes` and
    /// mark it with a takeover entry one second later.
    pub fn takeover(&mut self, minutes: u16) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        let data = &mut self.entries.data;
        match data.last().map(|e| &e.status) {
            Some(Status::End) => {}
            Some(Status::Takeover) => {
                return Err(TrackerError::TimeDataError {
                    message: "today was already taken over".to_owned(),
                });
            }
            _ => {
                return Err(TrackerError::TimeDataError {
                    message: "end today first to take over".to_owned(),
                });
            }
        }
        if minutes == 0 {
            return Err(TrackerError::TimeDataError {
                message: "takeover needs at least 1 minute".to_owned(),
            });
        }

        let n = data.len();
        let end = data[n - 1].time - Duration::minutes(minutes.into());
        if n < 2 || end <= data[n - 2].time {
            return Err(TrackerError::TimeDataError {
                message: format!(
                    "taking over {minutes} minutes would move the end to {} and collide with existing entries",
                    DateTime::<chrono::Local>::from(end).format("%H:%M")
                ),
            });
        }

        data[n - 1].time = end;
        log::debug!("moved end back to {}", end);
        self.append(Status::Takeover, end + Duration::seconds(1), Some("takeover".to_owned()))
    }

    pub fn read_from_file(&mut self) -> Result<&mut Self, TrackerError> {
        self.assert_build()?;
        if self.file.exists() {
            let f = File::open(&self.file)?;
            self.entries = Upgrade::upgrade(BufReader::new(f))?;

            self.entries.data.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap());
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
    use std::{ fs::{ self, File }, io::Write, ops::Add };

    use chrono::{ Duration, TimeZone, Timelike };

    use crate::{ Entry, Status, TimeData, TrackerError };

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

    mod fill_break {
        use chrono::{ DateTime, Utc };

        use crate::Entries;

        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<Utc> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap()
        }

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
                ..Default::default()
            }
        }

        fn summary(t: &TimeData) -> Vec<(u8, Status, DateTime<Utc>, Option<String>)> {
            t.entries.data
                .iter()
                .map(|e| (e.id, e.status.to_owned(), e.time, e.note.to_owned()))
                .collect()
        }

        fn filled() -> Option<String> {
            Some("filled mandatory remaining break".to_owned())
        }

        fn sec(t: DateTime<Utc>) -> DateTime<Utc> {
            t + Duration::seconds(1)
        }

        #[test]
        fn should_fill_after_start_without_break() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 16, 30)]);
            t.assert_break(Duration::minutes(30), Duration::zero())?;

            assert_eq!(
                vec![
                    (1, Status::Connect, at(8, 0), None),
                    (2, Status::Break, sec(at(8, 0)), filled()),
                    (3, Status::Connect, sec(at(8, 30)), None),
                    (4, Status::End, at(16, 30), None)
                ],
                summary(&t)
            );

            let settings = crate::Settings {
                limits: vec![crate::BreakLimit { start: 6 * 60, minutes: 30 }],
                ..Default::default()
            };
            let status = crate::StatusDaily::builder().data(t).settings(settings).build()?;
            assert_eq!("00:30", status.r#break.unwrap().to_string());
            assert_eq!("08:00", status.worktime.to_string());
            Ok(())
        }

        #[test]
        fn should_fill_after_real_start_behind_takeover() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 7, 30), (Status::Connect, 8, 0), (Status::End, 16, 30)]);
            t.assert_break(Duration::minutes(30), Duration::zero())?;

            assert_eq!((3, Status::Break, sec(at(8, 0)), filled()), summary(&t)[2]);
            assert_eq!((4, Status::Connect, sec(at(8, 30)), None), summary(&t)[3]);
            assert_eq!((5, Status::End, at(16, 30), None), summary(&t)[4]);
            Ok(())
        }

        #[test]
        fn should_extend_first_break() -> Result<(), TrackerError> {
            let mut t = day(
                &[
                    (Status::Connect, 8, 0),
                    (Status::Break, 12, 0),
                    (Status::Connect, 12, 15),
                    (Status::Break, 14, 0),
                    (Status::Connect, 14, 5),
                    (Status::End, 16, 30),
                ]
            );
            t.assert_break(Duration::minutes(30), Duration::minutes(20))?;

            assert_eq!(
                vec![
                    (1, Status::Connect, at(8, 0), None),
                    (2, Status::Break, at(12, 0), None),
                    (3, Status::Connect, at(12, 15), None),
                    (4, Status::Break, sec(at(12, 15)), filled()),
                    (5, Status::Connect, sec(at(12, 25)), None),
                    (6, Status::Break, at(14, 0), None),
                    (7, Status::Connect, at(14, 5), None),
                    (8, Status::End, at(16, 30), None)
                ],
                summary(&t)
            );
            Ok(())
        }

        #[test]
        fn should_skip_fill_overlapping_next_entry() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 8, 20)]);
            let before = summary(&t);
            t.assert_break(Duration::minutes(30), Duration::zero())?;

            assert_eq!(before, summary(&t));
            Ok(())
        }

        #[test]
        fn should_leave_end_for_takeover() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 17, 0)]);
            t.assert_break(Duration::minutes(30), Duration::zero())?.takeover(30)?;

            assert_eq!((4, Status::End, at(16, 30), None), summary(&t)[3]);
            Ok(())
        }
    }

    mod takeover {
        use chrono::{ DateTime, Utc };

        use crate::Entries;

        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<Utc> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap()
        }

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
                ..Default::default()
            }
        }

        fn summary(t: &TimeData) -> Vec<(u8, Status, DateTime<Utc>, Option<String>)> {
            t.entries.data
                .iter()
                .map(|e| (e.id, e.status.to_owned(), e.time, e.note.to_owned()))
                .collect()
        }

        #[test]
        fn should_move_end_back_and_mark_takeover() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 17, 0)]);
            t.takeover(30)?;

            assert_eq!(
                vec![
                    (1, Status::Connect, at(8, 0), None),
                    (2, Status::End, at(16, 30), None),
                    (3, Status::Takeover, at(16, 30) + Duration::seconds(1), Some("takeover".to_owned()))
                ],
                summary(&t)
            );
            Ok(())
        }

        #[test]
        fn should_reject_running_day() {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::Break, 12, 0)]);
            let before = summary(&t);
            assert!(t.takeover(30).is_err());
            assert_eq!(before, summary(&t));
        }

        #[test]
        fn should_reject_empty_day() {
            let mut t = day(&[]);
            assert!(t.takeover(30).is_err());
            assert!(t.entries.data.is_empty());
        }

        #[test]
        fn should_reject_second_takeover() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 17, 0)]);
            t.takeover(30)?;
            let before = summary(&t);
            assert!(t.takeover(10).is_err());
            assert_eq!(before, summary(&t));
            Ok(())
        }

        #[test]
        fn should_reject_collision_with_previous_entry() {
            // As left by `end` filling a missing break: connect one minute before the end.
            let mut t = day(
                &[
                    (Status::Connect, 8, 0),
                    (Status::Break, 16, 29),
                    (Status::Connect, 16, 59),
                    (Status::End, 17, 0),
                ]
            );
            let before = summary(&t);

            let err = t.takeover(30).unwrap_err().to_string();
            assert!(err.contains("collide with existing entries"), "{err}");
            assert_eq!(before, summary(&t));
        }

        #[test]
        fn should_reject_end_on_previous_entry() {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 8, 30)]);
            assert!(t.takeover(30).is_err());
        }
    }

    mod prepend {
        use chrono::{ DateTime, Utc };

        use crate::Entries;

        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<Utc> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap()
        }

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
                ..Default::default()
            }
        }

        fn summary(t: &TimeData) -> Vec<(u8, Status, DateTime<Utc>, Option<String>)> {
            t.entries.data
                .iter()
                .map(|e| (e.id, e.status.to_owned(), e.time, e.note.to_owned()))
                .collect()
        }

        fn added() -> Option<String> {
            Some("manually added".to_owned())
        }

        #[test]
        fn should_prepend_before_first_start() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::Break, 12, 0)]);
            t.prepend(30, "manually added")?;

            assert_eq!(
                vec![
                    (1, Status::Connect, at(7, 30), added()),
                    (2, Status::Connect, at(8, 0), None),
                    (3, Status::Break, at(12, 0), None)
                ],
                summary(&t)
            );
            Ok(())
        }

        #[test]
        fn should_prepend_to_ended_day_without_touching_breaks() -> Result<(), TrackerError> {
            let mut t = day(
                &[
                    (Status::Connect, 8, 0),
                    (Status::Break, 12, 0),
                    (Status::Connect, 12, 30),
                    (Status::End, 16, 30),
                ]
            );
            t.prepend(45, "manually added")?;

            assert_eq!(
                vec![
                    (1, Status::Connect, at(7, 15), added()),
                    (2, Status::Connect, at(8, 0), None),
                    (3, Status::Break, at(12, 0), None),
                    (4, Status::Connect, at(12, 30), None),
                    (5, Status::End, at(16, 30), None)
                ],
                summary(&t)
            );
            Ok(())
        }

        #[test]
        fn should_prepend_before_earlier_addition() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0)]);
            t.prepend(30, "manually added")?.prepend(15, "manually added")?;

            assert_eq!((1, Status::Connect, at(7, 15), added()), summary(&t)[0]);
            assert_eq!((2, Status::Connect, at(7, 30), added()), summary(&t)[1]);
            Ok(())
        }

        #[test]
        fn should_reject_without_start() {
            let mut t = day(&[]);
            assert!(t.prepend(30, "manually added").is_err());
            assert!(t.entries.data.is_empty());
        }

        #[test]
        fn should_reject_start_before_the_day() {
            let mut t = day(&[(Status::Connect, 1, 0)]);
            let before = summary(&t);

            assert!(t.prepend(61, "manually added").is_err());
            assert_eq!(before, summary(&t));
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
    }
}
