use chrono::{ DateTime, Duration, FixedOffset, NaiveDate, NaiveTime, TimeZone };
use serde::{ Deserialize, Serialize };

use crate::{ Entry, Settings, Status, StatusTime, TrackerError };

pub(crate) const CURRENT_VERSION: u8 = 4;


#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Timesheet {
    pub data: Vec<Entry>,
    pub version: u8,
}

impl Default for Timesheet {
    fn default() -> Self {
        Self { data: Vec::new(), version: CURRENT_VERSION }
    }
}

impl Timesheet {
    /// Time of the first connect of the day.
    pub fn start(&self) -> Option<StatusTime> {
        self.data.iter().find(|e| e.status == Status::Connect).map(StatusTime::from)
    }

    /// Time of the end of the day, `None` while the day is still running.
    pub fn end(&self) -> Option<StatusTime> {
        self.data.iter().find(|e| e.status == Status::End).map(StatusTime::from)
    }

    /// Total of the breaks that were ended by a connect.
    pub fn break_taken(&self) -> StatusTime {
        let mut taken = Duration::zero();
        let mut since = None;
        for e in &self.data {
            match (since, &e.status) {
                (None, Status::Break) => since = Some(e.time),
                (Some(start), Status::Connect) => {
                    taken = taken + (e.time - start);
                    since = None;
                }
                _ => {}
            }
        }
        log::debug!("a total of {:?} break duration was found", taken);
        taken.into()
    }

    /// Start of the first break of the day.
    pub fn first_break(&self) -> Option<DateTime<FixedOffset>> {
        self.data.iter().find(|e| e.status == Status::Break).map(|e| e.time)
    }

    /// Time from the start to the end of the day, or to `now` while the day is still running.
    pub fn online(&self, now: DateTime<FixedOffset>) -> Result<StatusTime, TrackerError> {
        let end = self.end_time().unwrap_or(now);
        Ok(StatusTime::from(end - self.start_time()?))
    }

    /// Target worktime of the day, by the weekday of its first entry.
    pub fn expected_worktime(&self, settings: &Settings) -> Result<StatusTime, TrackerError> {
        let first = self.data.first().ok_or_else(|| invalid("data not added to status"))?;
        let minutes = settings.workperday.minutes_on(first.time);
        Ok(StatusTime::from(Duration::minutes(minutes.into())))
    }

    /// Break to take for the day, based on whichever is higher, its target worktime or its
    /// online time less the threshold of the limits.
    pub fn expected_break(
        &self,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<StatusTime, TrackerError> {
        let online = self.online(now)?;
        let worktime = self.expected_worktime(settings)?;
        let (time, threshold) = if online >= worktime {
            (online.duration, Duration::minutes(settings.threshold_limits.into()))
        } else {
            (worktime.duration, Duration::zero())
        };
        let expected = settings.break_for(time - threshold);
        log::debug!("should take a break of {}", expected.num_minutes());
        Ok(expected.into())
    }

    /// When the target worktime is reached with the break taken or the expected break, whichever
    /// is longer.
    pub fn estimated_end(
        &self,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<DateTime<FixedOffset>, TrackerError> {
        let worktime = self.expected_worktime(settings)?;
        let r#break = self.longer_break(settings, now)?;
        Ok(self.start_time()? + worktime.duration + r#break.duration)
    }

    /// Time worked, the online time less the break taken or the expected break, whichever is
    /// longer.
    pub fn worktime(
        &self,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<StatusTime, TrackerError> {
        Ok(self.online(now)? - self.longer_break(settings, now)?)
    }

    /// Worktime above, or below if negative, the target worktime.
    pub fn overtime(
        &self,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<StatusTime, TrackerError> {
        Ok(self.worktime(settings, now)? - self.expected_worktime(settings)?)
    }

    fn longer_break(
        &self,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<StatusTime, TrackerError> {
        let taken = self.break_taken();
        let expected = self.expected_break(settings, now)?;
        Ok(if taken >= expected { taken } else { expected })
    }

    /// When the first connect of the day happened.
    fn start_time(&self) -> Result<DateTime<FixedOffset>, TrackerError> {
        if self.data.is_empty() {
            return Err(invalid("data not added to status"));
        }
        self.data
            .iter()
            .find(|e| e.status == Status::Connect)
            .map(|e| e.time)
            .ok_or_else(|| invalid("no initial connect found in data"))
    }

    /// When the first end of the day happened, `None` while the day is still running.
    fn end_time(&self) -> Option<DateTime<FixedOffset>> {
        self.data.iter().find(|e| e.status == Status::End).map(|e| e.time)
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
        if e_break <= a_break {
            return Ok(self);
        }
        let data = &self.data;
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

        let r#break = Entry::new(Status::Break, start, Some("filled mandatory remaining break".to_owned()));
        let connect = Entry::new(Status::Connect, end, None);
        log::debug!("fill break with {:?} and {:?}", r#break, connect);
        self.data.splice(anchor + 1..anchor + 1, [r#break, connect]);
        Ok(self)
    }

    pub fn append(
        &mut self,
        status: Status,
        time: DateTime<FixedOffset>,
        note: Option<String>
    ) -> Result<&mut Self, TrackerError> {
        let entry = Entry::new(status, time, note);
        log::debug!("append time data: {:?}", entry);
        self.data.append(&mut [entry].to_vec());
        Ok(self)
    }

    /// Fill the remaining target worktime of the day with an absence (`sick` or `vacation`).
    ///
    /// The absence starts at 08:00 of `date` in the timezone `zone` for an empty day, at the end
    /// of an ended day (whose end is replaced), and otherwise at `now`, or at the last entry if the
    /// day is not today. It lasts until the day's worktime reaches the target with its break.
    pub fn absence<Tz: TimeZone>(
        &mut self,
        zone: &Tz,
        date: NaiveDate,
        note: &str,
        now: DateTime<FixedOffset>,
        settings: &Settings
    ) -> Result<&mut Self, TrackerError> {
        let mut data = self.data.clone();
        let from = match data.last().map(|e| (e.status.to_owned(), e.time)) {
            None => Self::default_start(zone, date)?,
            Some((Status::End, time)) => {
                data.pop();
                time
            }
            Some((Status::Connect | Status::Break, time)) => {
                if now.date_naive() == date { now.max(time) } else { time }
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
        let target = Duration::minutes(settings.workperday.minutes_on(start).into());
        let end = start + target + taken.max(settings.break_for(target));
        if end <= from {
            return Err(TrackerError::TimeDataError {
                message: format!("target worktime already reached, not adding {note}"),
            });
        }

        self.data = data;
        self.append(Status::Connect, from, Some(note.to_owned()))?.append(Status::End, end, None)
    }

    /// 08:00 of `date` in the timezone `zone`, with the offset it has at that moment.
    fn default_start<Tz: TimeZone>(
        zone: &Tz,
        date: NaiveDate
    ) -> Result<DateTime<FixedOffset>, TrackerError> {
        let start = date.and_time(NaiveTime::from_hms_opt(8, 0, 0).unwrap());
        zone.from_local_datetime(&start)
            .earliest()
            .map(|start| start.fixed_offset())
            .ok_or_else(|| TrackerError::TimeDataError {
                message: format!("{start} doesn't exist in the local time"),
            })
    }

    /// Prepend a connect entry `minutes` before the first connect of the day.
    pub fn prepend(&mut self, minutes: u16, note: &str) -> Result<&mut Self, TrackerError> {
        let first = self.data
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

        let entry = Entry::new(Status::Connect, time, Some(note.to_owned()));
        log::debug!("prepend time data: {:?}", entry);
        self.data.insert(0, entry);
        Ok(self)
    }

    /// Take over `minutes` of an ended day to the next start: move its end back by `minutes` and
    /// mark it with a takeover entry one second later.
    pub fn takeover(&mut self, minutes: u16) -> Result<&mut Self, TrackerError> {
        let data = &mut self.data;
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
                    end.format("%H:%M")
                ),
            });
        }

        data[n - 1].time = end;
        log::debug!("moved end back to {}", end);
        self.append(Status::Takeover, end + Duration::seconds(1), Some("takeover".to_owned()))
    }
}

fn invalid(message: &str) -> TrackerError {
    TrackerError::StatusError { message: message.to_owned() }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::{ DateTime, Duration, Local, NaiveDate, TimeZone, Timelike };

    use crate::{ Entry, Status, Timesheet, TrackerError };

    fn logger() {
        // std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod fill_break {
        use chrono::{ DateTime, FixedOffset, Utc };


        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<FixedOffset> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap().fixed_offset()
        }

        fn day(entries: &[(Status, u32, u32)]) -> Timesheet {
            let data = entries
                .iter()
                .map(|(status, h, m)| Entry {
                    status: status.to_owned(),
                    time: at(*h, *m),
                    note: None,
                })
                .collect();
            Timesheet {
                data,
                ..Default::default()
            }
        }

        fn summary(t: &Timesheet) -> Vec<(Status, DateTime<FixedOffset>, Option<String>)> {
            t.data
                .iter()
                .map(|e| (e.status.to_owned(), e.time, e.note.to_owned()))
                .collect()
        }

        fn filled() -> Option<String> {
            Some("filled mandatory remaining break".to_owned())
        }

        fn sec(t: DateTime<FixedOffset>) -> DateTime<FixedOffset> {
            t + Duration::seconds(1)
        }

        #[test]
        fn should_fill_after_start_without_break() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 16, 30)]);
            t.assert_break(Duration::minutes(30), Duration::zero())?;

            assert_eq!(
                vec![
                    (Status::Connect, at(8, 0), None),
                    (Status::Break, sec(at(8, 0)), filled()),
                    (Status::Connect, sec(at(8, 30)), None),
                    (Status::End, at(16, 30), None)
                ],
                summary(&t)
            );

            let settings = crate::Settings {
                limits: vec![crate::BreakLimit { start: 6 * 60, minutes: 30 }],
                ..Default::default()
            };
            assert_eq!("00:30", t.break_taken().to_string());
            assert_eq!("08:00", t.worktime(&settings, chrono::Utc::now().fixed_offset())?.to_string());
            Ok(())
        }

        #[test]
        fn should_fill_after_real_start_behind_takeover() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 7, 30), (Status::Connect, 8, 0), (Status::End, 16, 30)]);
            t.assert_break(Duration::minutes(30), Duration::zero())?;

            assert_eq!((Status::Break, sec(at(8, 0)), filled()), summary(&t)[2]);
            assert_eq!((Status::Connect, sec(at(8, 30)), None), summary(&t)[3]);
            assert_eq!((Status::End, at(16, 30), None), summary(&t)[4]);
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
                    (Status::Connect, at(8, 0), None),
                    (Status::Break, at(12, 0), None),
                    (Status::Connect, at(12, 15), None),
                    (Status::Break, sec(at(12, 15)), filled()),
                    (Status::Connect, sec(at(12, 25)), None),
                    (Status::Break, at(14, 0), None),
                    (Status::Connect, at(14, 5), None),
                    (Status::End, at(16, 30), None)
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

            assert_eq!((Status::End, at(16, 30), None), summary(&t)[3]);
            Ok(())
        }
    }

    mod takeover {
        use chrono::{ DateTime, FixedOffset, Utc };


        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<FixedOffset> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap().fixed_offset()
        }

        fn day(entries: &[(Status, u32, u32)]) -> Timesheet {
            let data = entries
                .iter()
                .map(|(status, h, m)| Entry {
                    status: status.to_owned(),
                    time: at(*h, *m),
                    note: None,
                })
                .collect();
            Timesheet {
                data,
                ..Default::default()
            }
        }

        fn summary(t: &Timesheet) -> Vec<(Status, DateTime<FixedOffset>, Option<String>)> {
            t.data
                .iter()
                .map(|e| (e.status.to_owned(), e.time, e.note.to_owned()))
                .collect()
        }

        #[test]
        fn should_move_end_back_and_mark_takeover() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 17, 0)]);
            t.takeover(30)?;

            assert_eq!(
                vec![
                    (Status::Connect, at(8, 0), None),
                    (Status::End, at(16, 30), None),
                    (Status::Takeover, at(16, 30) + Duration::seconds(1), Some("takeover".to_owned()))
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
            assert!(t.data.is_empty());
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
        use chrono::{ DateTime, FixedOffset, Utc };


        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<FixedOffset> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap().fixed_offset()
        }

        fn day(entries: &[(Status, u32, u32)]) -> Timesheet {
            let data = entries
                .iter()
                .map(|(status, h, m)| Entry {
                    status: status.to_owned(),
                    time: at(*h, *m),
                    note: None,
                })
                .collect();
            Timesheet {
                data,
                ..Default::default()
            }
        }

        fn summary(t: &Timesheet) -> Vec<(Status, DateTime<FixedOffset>, Option<String>)> {
            t.data
                .iter()
                .map(|e| (e.status.to_owned(), e.time, e.note.to_owned()))
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
                    (Status::Connect, at(7, 30), added()),
                    (Status::Connect, at(8, 0), None),
                    (Status::Break, at(12, 0), None)
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
                    (Status::Connect, at(7, 15), added()),
                    (Status::Connect, at(8, 0), None),
                    (Status::Break, at(12, 0), None),
                    (Status::Connect, at(12, 30), None),
                    (Status::End, at(16, 30), None)
                ],
                summary(&t)
            );
            Ok(())
        }

        #[test]
        fn should_prepend_before_earlier_addition() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0)]);
            t.prepend(30, "manually added")?.prepend(15, "manually added")?;

            assert_eq!((Status::Connect, at(7, 15), added()), summary(&t)[0]);
            assert_eq!((Status::Connect, at(7, 30), added()), summary(&t)[1]);
            Ok(())
        }

        #[test]
        fn should_reject_without_start() {
            let mut t = day(&[]);
            assert!(t.prepend(30, "manually added").is_err());
            assert!(t.data.is_empty());
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
        use chrono::{ DateTime, FixedOffset, NaiveDate, Utc };

        use crate::{ BreakLimit, Settings };

        use super::*;

        fn at(hour: u32, minute: u32) -> DateTime<FixedOffset> {
            Utc.with_ymd_and_hms(2022, 2, 7, hour, minute, 0).unwrap().fixed_offset()
        }

        fn settings() -> Settings {
            Settings {
                limits: vec![BreakLimit { start: 6 * 60, minutes: 30 }],
                ..Default::default()
            }
        }

        /// Monday 2022-02-07 with the given (status, hour, minute) entries.
        fn day(entries: &[(Status, u32, u32)]) -> Timesheet {
            let data = entries
                .iter()
                .map(|(status, h, m)| Entry {
                    status: status.to_owned(),
                    time: at(*h, *m),
                    note: None,
                })
                .collect();
            Timesheet {
                data,
                ..Default::default()
            }
        }

        fn summary(t: &Timesheet) -> Vec<(Status, DateTime<FixedOffset>, Option<String>)> {
            t.data
                .iter()
                .map(|e| (e.status.to_owned(), e.time, e.note.to_owned()))
                .collect()
        }

        /// The day of `at`.
        fn date() -> NaiveDate {
            NaiveDate::from_ymd_opt(2022, 2, 7).unwrap()
        }

        fn worktime(t: &Timesheet) -> String {
            t.worktime(&settings(), at(23, 0)).unwrap().to_string()
        }

        fn sick() -> Option<String> {
            Some("sick".to_owned())
        }

        #[test]
        fn should_fill_empty_day() -> Result<(), TrackerError> {
            let mut t = day(&[]);
            t.absence(&Utc, date(), "sick", at(9, 0), &settings())?;

            // Starts at 08:00 of the day in the given timezone, and ends after the target and the
            // break.
            assert_eq!(
                vec![(Status::Connect, at(8, 0), sick()), (Status::End, at(16, 30), None)],
                summary(&t)
            );
            assert_eq!("08:00", worktime(&t));
            Ok(())
        }

        #[test]
        fn should_extend_running_day_from_now() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0)]);
            t.absence(&Utc, date(), "sick", at(10, 0), &settings())?;

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
            t.absence(&Utc, date(), "sick", at(15, 0), &settings())?;

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
            t.absence(&Utc, date(), "sick", at(14, 0), &settings())?;

            assert_eq!((Status::End, at(17, 0), None), summary(&t)[4]);
            assert_eq!("08:00", worktime(&t));
            Ok(())
        }

        #[test]
        fn should_end_running_break() -> Result<(), TrackerError> {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::Break, 12, 0)]);
            t.absence(&Utc, date(), "sick", at(12, 15), &settings())?;

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
            t.absence(&Utc, date(), "sick", Utc.with_ymd_and_hms(2022, 2, 9, 9, 0, 0).unwrap().fixed_offset(), &settings())?;

            assert_eq!((Status::Connect, at(8, 0), sick()), summary(&t)[1]);
            assert_eq!((Status::End, at(16, 30), None), summary(&t)[2]);
            Ok(())
        }

        #[test]
        fn should_not_change_day_with_reached_target() {
            let mut t = day(&[(Status::Connect, 6, 0), (Status::End, 14, 30)]);
            let before = summary(&t);

            assert!(t.absence(&Utc, date(), "vacation", at(15, 0), &settings()).is_err());
            assert_eq!(before, summary(&t));
        }

        #[test]
        fn should_not_change_day_ending_with_takeover() {
            let mut t = day(&[(Status::Connect, 8, 0), (Status::End, 10, 0), (Status::Takeover, 10, 0)]);
            let before = summary(&t);

            assert!(t.absence(&Utc, date(), "sick", at(11, 0), &settings()).is_err());
            assert_eq!(before, summary(&t));
        }
    }

    mod assert_break {
        use super::*;

        fn timesheet(json: &str) -> Timesheet {
            serde_json::from_str(json).unwrap()
        }

        #[test]
        fn should_do_nothing_with_long_break() -> Result<(), TrackerError> {
            logger();
            let mut time_data = timesheet(
                "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"status\":\"Break\",\"time\":\"2022-08-04T00:30:53.523319900Z\"},{\"status\":\"Connect\",\"time\":\"2022-08-04T02:15:53.523319900Z\"},{\"status\":\"End\",\"time\":\"2022-08-04T08:00:53.523332900Z\"}],\"version\":1}"
            );

            time_data.assert_break(
                Duration::minutes(45),
                Duration::hours(1).add(Duration::minutes(15))
            )?;
            assert_eq!(4, time_data.data.len());

            let r#break = time_data.data.iter().find(|x| x.status == Status::Break).unwrap();
            let connect = time_data.data
                .iter()
                .filter(|x| x.status == Status::Connect)
                .last()
                .unwrap();

            let duration_b = r#break.time.num_seconds_from_midnight();
            let duration_c = connect.time.num_seconds_from_midnight();
            assert_eq!(6300, duration_c - duration_b, "total break duration should be 30 minutes");
            Ok(())
        }

        #[test]
        fn should_do_nothing_with_enough_break() -> Result<(), TrackerError> {
            logger();
            let mut time_data = timesheet(
                "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-08-04T00:00:53.523319900Z\"},{\"status\":\"End\",\"time\":\"2022-08-04T04:00:53.523332900Z\"}],\"version\":1}"
            );

            time_data.assert_break(Duration::minutes(15), Duration::minutes(45))?;
            assert_eq!(2, time_data.data.len());
            Ok(())
        }
    }

    mod calculation {
        use std::ops::Sub;

        use chrono::{ DateTime, FixedOffset, Utc };

        use crate::{ BreakLimit, Settings, WorkPerDay };

        use super::*;

        /// The start of the days of the tests.
        fn midnight() -> DateTime<FixedOffset> {
            Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap().fixed_offset()
        }

        /// Clock for days that are ended, so it doesn't matter.
        fn now() -> DateTime<FixedOffset> {
            Utc.with_ymd_and_hms(2022, 2, 2, 12, 0, 0).unwrap().fixed_offset()
        }

        #[test]
        fn should_set_connect() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 15, 6, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };
            assert!(data.start().is_some());
        }

        #[test]
        fn no_connect() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            assert!(data.worktime(&Settings::default(), now()).is_err());
        }

        #[test]
        fn should_set_end() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 10, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 10, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 10, 10, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            assert!(data.end().is_some());
            assert_eq!(10, data.end().unwrap().duration.num_hours())
        }

        #[test]
        fn should_use_now_as_end_of_running_day() {
            let data = Timesheet {
                data: vec![
                    Entry {
                        status: Status::Connect,
                        time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap().fixed_offset(),
                        note: None,
                    },
                ],
                ..Default::default()
            };
            let now = Utc.with_ymd_and_hms(2022, 2, 2, 9, 30, 0).unwrap().fixed_offset();

            assert!(data.end().is_none());
            let online = data.online(now).unwrap();
            assert_eq!(1, online.hours());
            assert_eq!(30, online.minutes());
        }

        #[test]
        fn should_calculate_break() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 10, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 40, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let r#break = data.break_taken();
            assert_eq!(0, r#break.hours());
            assert_eq!(5, r#break.minutes());
        }

        #[test]
        fn should_calculate_break_between_mutliple_breaks() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 10, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 40, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 40, 5)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 9, 40, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 9, 40, 55)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 9, 55, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 10, 0, 55)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 10, 0, 56)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 10, 1, 56)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let r#break = data.break_taken();
            assert_eq!(0, r#break.hours());
            assert_eq!(21, r#break.minutes());
        }

        #[test]
        fn should_calculate_online_time() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 15, 6, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let online = data.online(now()).unwrap();

            assert_eq!(7, online.hours());
            assert_eq!(3, online.minutes());
        }

        #[test]
        fn should_calculate_est_end_with_break_fully_taken() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 5, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };
            let est_end = data.estimated_end(&settings, now()).unwrap();
            let expected_end = Duration::hours(8).add(Duration::minutes(45));
            assert_eq!(expected_end, (est_end - midnight()));
        }

        #[test]
        fn should_calculate_est_end_with_break_not_fully_taken() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 0, 00, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 15, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 5, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let est_end = data.estimated_end(&settings, now()).unwrap();

            let expected_end = Duration::hours(8).add(Duration::minutes(45));
            assert_eq!(expected_end, (est_end - midnight()));
        }

        #[test]
        fn should_calculate_est_end_with_more_break_taken() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 0, 00, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 4, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 5, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let est_end = data.estimated_end(&settings, now()).unwrap();
            let expected_end = Duration::hours(9);
            assert_eq!(expected_end, (est_end - midnight()));
        }

        #[test]
        fn should_calculate_est_end_with_no_break_taken() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 10, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 13, 0, 1)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 13, 15, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 15, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let est_end = data.estimated_end(&settings, now()).unwrap();
            let expected_end = Duration::hours(10).add(
                Duration::hours(8).add(Duration::minutes(45))
            );
            assert_eq!(expected_end, (est_end - midnight()));
        }

        #[test]
        fn should_calculate_est_end_with_no_break_taken_short_day_and_threshold() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 10, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 13, 0, 1)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 13, 20, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 15, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    wednesday: 6 * 60,
                    ..Default::default()
                },
                threshold_limits: 5,
                ..Default::default()
            };

            let est_end = data.estimated_end(&settings, now()).unwrap();
            let expected_end = Duration::hours(10).add(Duration::hours(6));
            assert_eq!(
                expected_end,
                (est_end - midnight()),
                "expected end at 6:00 but got {}",
                est_end
            );
        }

        #[test]
        fn should_calculate_est_end_with_no_break_taken_odd_day() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 10, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Disconnect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 13, 0, 1)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 13, 20, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 15, 45, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    wednesday: 7 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let est_end = data.estimated_end(&settings, now()).unwrap();
            let expected_end = Duration::hours(10).add(
                Duration::hours(7).add(Duration::minutes(30))
            );
            assert_eq!(expected_end, (est_end - midnight()));
        }

        #[test]
        fn should_calculate_no_break_on_short_day() {
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 22, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 16, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    wednesday: 6 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let exp_break = data.expected_break(&settings, now()).unwrap();
            let est_end = data.estimated_end(&settings, now()).unwrap();
            let expected_end = Duration::hours(14).add(Duration::minutes(22));
            assert_eq!(
                expected_end,
                (est_end - midnight()),
                "expected to end at 14:22 but got {}",
                est_end
            );
            assert_eq!(
                Duration::minutes(0),
                exp_break.duration,
                "expected break to be 00:00 but got {}",
                exp_break
            );
        }

        #[test]
        fn should_calculate_overtime() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 8, 55, 46)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 8, 56, 15)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 12, 25, 57)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 12, 26, 46)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 12, 28, 7)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 12, 58, 7)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 17, 0, 7)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 17, 15, 7)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 18, 27, 40)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 30,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    monday: 510,
                    ..Default::default()
                },
                ..Default::default()
            };

            let overtime = data.overtime(&settings, now()).unwrap();


            assert_eq!(
                Duration::minutes(16).add(Duration::seconds(5)),
                overtime.duration,
                "expected 0:16 overtime but was {}",
                overtime
            );
        }

        #[test]
        fn should_calculate_negative_overtime() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 8, 22, 11)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 7, 12, 16, 32)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 0,
                        minutes: 15,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    monday: 360,
                    ..Default::default()
                },
                ..Default::default()
            };

            let overtime = data.overtime(&settings, now()).unwrap();


            assert_eq!(
                Duration::hours(-2).sub(Duration::minutes(21).sub(Duration::seconds(21))),
                overtime.duration,
                "expected -2:21 overtime but was {}",
                overtime
            );
        }

        #[test]
        fn should_calculate_worktime_and_expected_break_with_only_30_minutes_break_taken() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 30, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 17, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                ..Default::default()
            };

            let exp_break = data.expected_break(&settings, now()).unwrap();
            let worktime = data.worktime(&settings, now()).unwrap();

            assert_eq!(
                Duration::minutes(45),
                exp_break.duration,
                "expected 45 minutes break but was {}",
                exp_break
            );
            assert_eq!(
                Duration::hours(8).add(Duration::minutes(15)),
                worktime.duration,
                "expected 8:15 working time but was {}",
                worktime
            )
        }

        #[test]
        fn should_calculate_worktime_and_expected_break_with_4_hours_break_taken() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 16, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 21, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                ..Default::default()
            };

            let exp_break = data.expected_break(&settings, now()).unwrap();
            let worktime = data.worktime(&settings, now()).unwrap();

            assert_eq!(
                Duration::minutes(60),
                exp_break.duration,
                "expected 1:00 break but was {}",
                exp_break
            );
            assert_eq!(
                Duration::hours(9),
                worktime.duration,
                "expected 9:00 working time but was {}",
                worktime
            )
        }

        #[test]
        fn should_calculate_worktime_and_expected_break_with_4_hours_break_taken_and_over_10_hours_worktime() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 16, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 23, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                ..Default::default()
            };

            let exp_break = data.expected_break(&settings, now()).unwrap();
            let worktime = data.worktime(&settings, now()).unwrap();

            assert_eq!(
                Duration::hours(1),
                exp_break.duration,
                "expected 1:00 hour break but was {}",
                exp_break
            );
            assert_eq!(
                Duration::hours(11),
                worktime.duration,
                "expected 11:00 working time but was {}",
                worktime
            )
        }

        #[test]
        fn should_calculate_worktime_and_no_expected_break_because_less_than_6_hours_worktime() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 4, 8, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 4, 14, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    friday: 6 * 60,
                    ..Default::default()
                },
                threshold_limits: 5,
                ..Default::default()
            };

            let exp_break = data.expected_break(&settings, now()).unwrap();
            let worktime = data.worktime(&settings, now()).unwrap();

            assert_eq!(
                Duration::minutes(0),
                exp_break.duration,
                "expected 0:00 hour break but was {}",
                exp_break
            );
            assert_eq!(
                Duration::hours(6),
                worktime.duration,
                "expected 6:00 working time but was {}",
                worktime
            )
        }

        #[test]
        fn expected_break_not_displayed_correctly_in_status() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 14, 0, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                        ].to_vec()
                },
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                    BreakLimit {
                        start: 10 * 60,
                        minutes: 60,
                    },
                ].to_vec(),
                workperday: WorkPerDay {
                    friday: 6 * 60,
                    ..Default::default()
                },
                threshold_limits: 5,
                ..Default::default()
            };

            let exp_break = data.expected_break(&settings, now()).unwrap();

            assert_eq!(
                Duration::minutes(45),
                exp_break.duration,
                "expected 0:45 hour break but was {}",
                exp_break
            );
        }
    }

    mod local_time {
        use chrono::FixedOffset;

        use crate::Settings;

        use super::*;

        /// The time `hour`:`minute` of the day `month`/`day` of 2022 at `offset` hours from UTC.
        fn at(offset: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<FixedOffset> {
            FixedOffset::east_opt(offset * 3600)
                .unwrap()
                .with_ymd_and_hms(2022, month, day, hour, minute, 0)
                .unwrap()
        }

        fn timesheet(entries: &[(Status, DateTime<FixedOffset>)]) -> Timesheet {
            let data = entries
                .iter()
                .map(|(status, time)| Entry { status: status.to_owned(), time: *time, note: None })
                .collect();
            Timesheet { data, ..Default::default() }
        }

        #[test]
        fn should_measure_online_time_across_a_timezone_change() {
            // 08:00 in Berlin to 17:00 in New York is 07:00 to 22:00 UTC.
            let t = timesheet(
                &[(Status::Connect, at(1, 2, 7, 8, 0)), (Status::End, at(-5, 2, 7, 17, 0))]
            );

            assert_eq!("15:00", t.online(at(0, 2, 8, 0, 0)).unwrap().to_string());
        }

        #[test]
        fn should_measure_breaks_across_a_timezone_change() {
            let t = timesheet(
                &[
                    (Status::Connect, at(1, 2, 7, 8, 0)),
                    (Status::Break, at(1, 2, 7, 12, 0)),
                    // 11:30 in New York is 17:30 in Berlin, 5 hours and 30 minutes later.
                    (Status::Connect, at(-5, 2, 7, 11, 30)),
                ]
            );

            assert_eq!("05:30", t.break_taken().to_string());
        }

        #[test]
        fn should_measure_online_time_across_daylight_saving() {
            // On the 27th of March 2022 the clocks in Berlin go from 02:00 to 03:00, so 01:00 to
            // 04:00 is two hours.
            let t = timesheet(
                &[(Status::Connect, at(1, 3, 27, 1, 0)), (Status::End, at(2, 3, 27, 4, 0))]
            );

            assert_eq!("02:00", t.online(at(2, 3, 28, 0, 0)).unwrap().to_string());
        }

        #[test]
        fn should_measure_a_day_crossing_midnight() {
            let t = timesheet(
                &[(Status::Connect, at(1, 2, 7, 22, 0)), (Status::End, at(1, 2, 8, 2, 0))]
            );

            assert_eq!("04:00", t.online(at(1, 2, 8, 12, 0)).unwrap().to_string());
        }

        #[test]
        fn should_end_a_running_day_crossing_midnight_at_now() {
            let t = timesheet(&[(Status::Connect, at(1, 2, 7, 22, 0))]);

            assert_eq!("03:30", t.online(at(1, 2, 8, 1, 30)).unwrap().to_string());
        }

        #[test]
        fn should_estimate_the_end_of_a_day_crossing_midnight() {
            // Monday has a target of 8 hours, which are reached on Tuesday at 06:00.
            let t = timesheet(&[(Status::Connect, at(1, 2, 7, 22, 0))]);
            let end = t.estimated_end(&Settings::default(), at(1, 2, 8, 1, 0)).unwrap();

            assert_eq!(at(1, 2, 8, 6, 0), end);
            assert_eq!("06:00", end.format("%H:%M").to_string());
        }

        #[test]
        fn should_take_the_weekday_target_from_the_local_date() {
            // 00:30 on Monday in Tokyo is still Sunday in UTC, but the target is the one of Monday.
            let t = timesheet(
                &[(Status::Connect, at(9, 2, 7, 0, 30)), (Status::End, at(9, 2, 7, 8, 30))]
            );
            let now = at(9, 2, 7, 12, 0);

            assert_eq!("08:00", t.expected_worktime(&Settings::default()).unwrap().to_string());
            assert_eq!("00:00", t.overtime(&Settings::default(), now).unwrap().to_string());
        }

        #[test]
        fn should_take_the_weekday_target_of_a_day_that_is_the_day_before_in_utc() {
            // 20:00 on Monday in Los Angeles is already Tuesday in UTC.
            let t = timesheet(
                &[(Status::Connect, at(-8, 2, 7, 20, 0)), (Status::End, at(-8, 2, 7, 23, 0))]
            );
            let now = at(-8, 2, 8, 12, 0);

            assert_eq!("-05:00", t.overtime(&Settings::default(), now).unwrap().to_string());
        }

        #[test]
        fn should_prepend_within_the_local_day() {
            // 00:30 on Monday in Tokyo is Sunday 15:30 in UTC, 20 minutes earlier is still Monday.
            let mut t = timesheet(&[(Status::Connect, at(9, 2, 7, 0, 30))]);

            assert!(t.prepend(20, "takeover").is_ok());
            assert_eq!(at(9, 2, 7, 0, 10), t.data[0].time);
        }

        #[test]
        fn should_not_prepend_into_the_previous_local_day() {
            // 40 minutes earlier is Sunday 23:50 in Tokyo, although it is the same date in UTC.
            let mut t = timesheet(&[(Status::Connect, at(9, 2, 7, 0, 30))]);
            let before = t.data.len();

            assert!(t.prepend(40, "takeover").is_err());
            assert_eq!(before, t.data.len());
        }

        #[test]
        fn should_not_take_over_into_the_previous_local_day() {
            let mut t = timesheet(
                &[
                    (Status::Connect, at(9, 2, 7, 0, 30)),
                    (Status::End, at(9, 2, 7, 8, 30)),
                ]
            );

            assert!(t.takeover(60).is_ok());
            assert_eq!(at(9, 2, 7, 7, 30), t.data[1].time);
        }

        #[test]
        fn should_start_an_absence_at_eight_in_the_given_zone() {
            for (offset, hours) in [(9, "+09:00"), (-11, "-11:00"), (1, "+01:00"), (2, "+02:00")] {
                let zone = FixedOffset::east_opt(offset * 3600).unwrap();
                let date = NaiveDate::from_ymd_opt(2022, 2, 7).unwrap();
                let mut t = Timesheet::default();
                t.absence(&zone, date, "sick", at(0, 1, 1, 0, 0), &Settings::default()).unwrap();

                assert_eq!(at(offset, 2, 7, 8, 0), t.data[0].time, "{hours}");
                assert_eq!(at(offset, 2, 7, 16, 0), t.data[1].time, "{hours}");
            }
        }

        #[test]
        fn should_start_an_absence_at_eight_local_time_with_the_offset_of_that_day() {
            // On the 7th of February the machine has the offset of its winter time, on the 8th of
            // August the one of its summer time, whatever the offsets are.
            for date in [
                NaiveDate::from_ymd_opt(2022, 2, 7).unwrap(),
                NaiveDate::from_ymd_opt(2022, 8, 8).unwrap(),
            ] {
                let mut t = Timesheet::default();
                t.absence(&Local, date, "sick", at(0, 1, 1, 0, 0), &Settings::default()).unwrap();

                let start = t.data[0].time;
                assert_eq!(date, start.date_naive());
                assert_eq!("08:00", start.format("%H:%M").to_string());
                let local = Local
                    .from_local_datetime(&date.and_hms_opt(8, 0, 0).unwrap())
                    .earliest()
                    .unwrap();
                assert_eq!(local.fixed_offset(), start);
            }
        }
    }
}
