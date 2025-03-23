use std::ops::{ Add, Mul };

use chrono::{ DateTime, Duration, Local };
use colored::Colorize;

use crate::{ Settings, Status, StatusTime, TimeData, TrackerError };

#[derive(Default, Clone, Debug)]
pub struct StatusDaily {
    data: Option<TimeData>,
    settings: Option<Settings>,
    pub start: Option<StatusTime>,
    pub end: Option<StatusTime>,
    temp_end: Option<StatusTime>,
    /// first break of day
    f_break: Option<DateTime<Local>>,
    /// calculated break. whichever is higher break or exp_break.
    calc_break: Option<StatusTime>,
    pub r#break: Option<StatusTime>,
    pub exp_break: Option<StatusTime>,

    online: Option<StatusTime>,
    est_end: StatusTime,

    pub worktime: StatusTime,
    /// expected worktime for that day.
    exp_worktime: Option<StatusTime>,
    pub overtime: StatusTime,
}

impl StatusDaily {
    pub fn builder() -> StatusDailyBuilder {
        StatusDailyBuilder {
            inner: StatusDaily::default(),
        }
    }

    fn has_connect(&self) -> bool {
        log::debug!("check if any connect entry is present");
        match self.data.as_ref() {
            Some(d) => d.entries.iter().any(|x| x.status == Status::Connect),
            None => false,
        }
    }

    fn set_start(&mut self) -> &mut Self {
        self.start = match
            self.data
                .as_ref()
                .unwrap()
                .entries.iter()
                .find(|x| x.status == Status::Connect)
        {
            Some(c) => {
                log::info!("connect at: {}", c.time.time());
                Some(c.into())
            }
            None => {
                log::error!("connect entry not found in time data");
                None
            }
        };
        self
    }

    fn set_end(&mut self) -> &mut Self {
        // set end time
        match
            self.data
                .as_ref()
                .unwrap()
                .entries.iter()
                .find(|x| x.status == Status::End)
        {
            Some(c) => {
                log::info!("end at: {}", c.time.time());
                log::info!("finished reading time data for {}", c.time.date_naive());
                self.end = Some(c.into());
            }
            None => {
                log::debug!("no end entry found threrefore create a temporary one");
                self.temp_end = Some(StatusTime::now());
            }
        }
        self
    }

    fn set_break(&mut self) -> &mut Self {
        // set currently taken break
        let mut break_duration = Duration::seconds(0);

        // temporary break datetime
        let mut tb: DateTime<Local> = DateTime::default();
        // has break
        let mut b = false;
        // first break set
        let mut f = false;
        let d = self.data.as_ref().unwrap();
        for n in 0..d.entries.len() {
            if !b {
                // get break entry
                if d.entries[n].status == Status::Break {
                    // temp save time
                    tb = d.entries[n].time;
                    log::info!("break at: {}", tb.time());
                    b = true;
                    if !f {
                        self.f_break = Some(d.entries[n].time);
                        f = true;
                    }
                }
            } else if b && d.entries[n].status == Status::Connect {
                // get next connect
                let tc = d.entries[n].time;
                log::info!("connect at: {}", tc.time());
                // caluclate time between both
                let tbd = tc - tb;
                // add to general break
                break_duration = break_duration + tbd;
                b = false;
            }
        }
        log::debug!("a total of {:?} break duration was found", break_duration);
        self.r#break = Some(break_duration.into());
        self
    }

    fn set_exp_break(&mut self) -> &mut Self {
        if self.online.is_none() {
            log::error!("online time not yet calculated");
        } else {
            let d = self.data.as_ref().unwrap();
            let s = self.settings.as_ref().unwrap();
            // get work per day as based on the first entry of time data
            let w = Duration::minutes(s.workperday.from(d.entries[0].time).to_owned().into());

            self.exp_worktime = Some(StatusTime::from(w));

            let o = self.online.as_ref().unwrap();

            // get whatever time is heigher, either expected working time for the day or the online time.
            let (tft, t) = if o >= &StatusTime::from(w) {
                (o.to_owned().duration, Duration::minutes(s.threshold_limits.to_owned().into()))
            } else {
                (w, Duration::seconds(0))
            };

            let mut bl = s.limits.to_owned();
            bl.sort_by(|x, y| y.start.partial_cmp(&x.start).unwrap());

            self.exp_break = match
                bl.iter_mut().find(|x| tft - t >= Duration::minutes(x.start.to_owned().into()))
            {
                Some(eb) => {
                    log::debug!("should take a break of {}", eb.minutes);
                    Some(Duration::minutes(eb.minutes.into()).into())
                }
                None => {
                    log::debug!("should not take a break");
                    Some(Duration::minutes(0).into())
                }
            };
        }
        self
    }

    fn set_calc_break(&mut self) -> &mut Self {
        if self.r#break.is_none() || self.exp_break.is_none() {
            log::error!("break times are not set");
        } else {
            let r#break = if self.r#break.as_ref().unwrap() >= self.exp_break.as_ref().unwrap() {
                self.r#break.as_ref().unwrap().to_owned()
            } else {
                self.exp_break.as_ref().unwrap().to_owned()
            };
            self.calc_break = Some(r#break);
        }
        self
    }

    fn set_online(&mut self) -> &mut Self {
        let end = if self.end.is_some() {
            self.end.to_owned().unwrap()
        } else {
            self.temp_end.to_owned().unwrap()
        };
        if self.start.is_none() {
            log::error!("start time is not set");
        } else {
            self.online = Some(end - self.start.to_owned().unwrap());
        }
        self
    }

    fn set_est_end(&mut self) -> &mut Self {
        if self.start.is_none() || self.exp_break.is_none() {
            log::error!("start and/or expected break times are not set");
        } else {
            // set expected end time
            let d = self.data.as_ref().unwrap();
            let s = self.settings.as_ref().unwrap();
            // get work per day as based on the first entry of time data
            let w = Duration::minutes(s.workperday.from(d.entries[0].time).to_owned().into());

            let e = if self.r#break.to_owned().unwrap() > self.exp_break.to_owned().unwrap() {
                w.add(self.r#break.to_owned().unwrap().into())
            } else {
                w.add(self.exp_break.to_owned().unwrap().into())
            };
            self.est_end = self.start.to_owned().unwrap().add(e.into());
        }
        self
    }

    fn set_worktime(&mut self) -> &mut Self {
        self.worktime =
            self.online.as_ref().unwrap().to_owned() - self.calc_break.to_owned().unwrap();
        self
    }

    fn set_overtime(&mut self) -> &mut Self {
        self.overtime = self.worktime.to_owned() - self.exp_worktime.as_ref().unwrap().to_owned();
        self
    }
}

pub struct StatusDailyBuilder {
    inner: StatusDaily,
}

impl StatusDailyBuilder {
    pub fn data(&mut self, data: TimeData) -> &mut Self {
        self.inner.data = Some(data);
        self
    }

    pub fn settings(&mut self, settings: Settings) -> &mut Self {
        self.inner.settings = Some(settings);
        self
    }

    pub fn build(&self) -> Result<StatusDaily, TrackerError> {
        if self.inner.data.is_none() || self.inner.data.as_ref().unwrap().entries.is_empty() {
            return Err(TrackerError::StatusError {
                message: "data not added to status".to_owned(),
            });
        }

        if self.inner.settings.is_none() {
            return Err(TrackerError::StatusError {
                message: "settings not added to status".to_owned(),
            });
        }

        if !self.inner.has_connect() {
            return Err(TrackerError::StatusError {
                message: "no initial connect found in data".to_owned(),
            });
        }

        let mut d = self.inner.to_owned();

        d.set_start()
            .set_end()
            .set_online()
            .set_break()
            .set_exp_break()
            .set_calc_break()
            .set_est_end()
            .set_worktime()
            .set_overtime();
        Ok(d)
    }

    pub fn empty_with_overtime(&mut self, overtime: StatusTime) -> StatusDaily {
        StatusDaily {
            overtime,
            ..Default::default()
        }
    }
}

impl std::fmt::Display for StatusDaily {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let start = self.start.to_owned().unwrap();
        let end = self.end.to_owned();
        let temp_end = self.temp_end.to_owned();
        let r#break = self.r#break.to_owned().unwrap();

        let zero_dr = Duration::minutes(0);
        let worktime = self.worktime.to_owned();
        let remaining = self.overtime.to_owned();

        let rm_fmt = match remaining.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Greater => format!("+{}", remaining).bright_green(),
            std::cmp::Ordering::Equal => format!("+{}", remaining).normal(),
            std::cmp::Ordering::Less => format!("-{}", remaining.mul(-1)).bright_red(),
        };

        let break_diff = self.exp_break.to_owned().unwrap() - self.r#break.to_owned().unwrap();

        let bk_fmt = match break_diff.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Less => format!("+{}", break_diff.mul(-1)).bright_yellow(),
            std::cmp::Ordering::Equal => format!("+{}", break_diff).normal(),
            std::cmp::Ordering::Greater => format!("-{}", break_diff).bright_red(),
        };

        let mut fmt_break_report = "".to_owned();

        let end_fmt = if end.is_some() {
            let f_break = self.f_break.map(StatusTime::from);

            if f_break.is_some() {
                let e_break = StatusTime::from(
                    self.f_break.unwrap().add(self.r#break.to_owned().unwrap().duration)
                );
                fmt_break_report = format!(
                    "\n{:width$}{} - {}",
                    "Break taken:",
                    f_break.unwrap(),
                    e_break,
                    width = 13
                );
            }

            format!("{}", end.unwrap()).bright_green()
        } else if temp_end.is_some() && temp_end.as_ref().unwrap() >= &self.est_end {
            format!("{}", temp_end.unwrap()).bright_green()
        } else {
            let hours = self.est_end.hours % 24;
            // This hack is required because in the relative time is know in the current context.
            // A time format like 25:15 doesn't make sense here, whereas 01:15 is understandable in this context.
            format!("{:0>2}:{:0>2} (est.)", hours, self.est_end.minutes).bright_yellow()
        };

        let line1 = format!("{:width$}{} ({})", "Work time:", worktime, rm_fmt, width = 13);
        let line2 = format!(
            "{:width$}{}",
            "Online time:",
            self.online.as_ref().unwrap(),
            width = 13
        );
        let line3 = format!("{:width$}{} ({})", "Break:", r#break, bk_fmt, width = 13);
        let line4 = fmt_break_report;
        let line5 = format!("{:width$}{}", "Started:", start, width = 13);
        let line6 = format!("{:width$}{}", "End:", end_fmt, width = 13);
        write!(f, "{}\n{}\n{}\n{}\n{}\n{}", line1, line2, line3, line4, line5, line6)
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use chrono::{ DateTime, Duration, Local, TimeZone };

    use crate::{
        BreakLimit,
        Entry,
        Settings,
        Status,
        StatusDaily,
        StatusTime,
        TimeData,
        WorkPerDayInMinutes,
    };

    use indoc::indoc;

    use colored::control::ShouldColorize;

    use std::ops::Sub;

    fn logger() {
        // std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod display {
        use super::*;

        #[test]
        fn status_daily_with_remaing_worktime_and_break() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 23, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 14, 45, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };
            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                ].to_vec(),
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            if ShouldColorize::from_env().should_colorize() {
                assert_eq!(
                    indoc!(
                        "Work time:   06:12 (\u{1b}[91m-01:48\u{1b}[0m)
                    Online time: 06:42
                    Break:       00:20 (\u{1b}[91m-00:10\u{1b}[0m)

                    Break taken: 12:03 - 12:23
                    Started:     08:03
                    End:         \u{1b}[92m14:45\u{1b}[0m"
                    ),
                    format!("{}", status)
                );
            } else {
                assert_eq!(
                    indoc!(
                        "Work time:   06:12 (-01:48)
                    Online time: 06:42
                    Break:       00:20 (-00:10)

                    Break taken: 12:03 - 12:23
                    Started:     08:03
                    End:         14:45"
                    ),
                    format!("{}", status)
                );
            }
        }

        #[test]
        fn status_daily_with_overtime_and_more_break() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 43, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 17, 45, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };
            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                ].to_vec(),
                ..Default::default()
            };
            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            if ShouldColorize::from_env().should_colorize() {
                assert_eq!(
                    indoc!(
                        "Work time:   09:02 (\u{1b}[92m+01:02\u{1b}[0m)
                    Online time: 09:42
                    Break:       00:40 (\u{1b}[93m+00:10\u{1b}[0m)

                    Break taken: 12:03 - 12:43
                    Started:     08:03
                    End:         \u{1b}[92m17:45\u{1b}[0m"
                    ),
                    format!("{}", status)
                );
            } else {
                assert_eq!(
                    indoc!(
                        "Work time:   09:02 (+01:02)
                    Online time: 09:42
                    Break:       00:40 (+00:10)

                    Break taken: 12:03 - 12:43
                    Started:     08:03
                    End:         17:45"
                    ),
                    format!("{}", status)
                );
            }
        }

        #[test]
        fn status_daily_on_point() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 33, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 16, 33, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };
            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                ].to_vec(),
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            if ShouldColorize::from_env().should_colorize() {
                assert_eq!(
                    indoc!(
                        "Work time:   08:00 (+00:00)
                        Online time: 08:30
                        Break:       00:30 (+00:00)

                        Break taken: 12:03 - 12:33
                        Started:     08:03
                        End:         \u{1b}[92m16:33\u{1b}[0m"
                    ),
                    format!("{}", status)
                );
            } else {
                assert_eq!(
                    indoc!(
                        "Work time:   08:00 (+00:00)
                    Online time: 08:30
                    Break:       00:30 (+00:00)

                    Break taken: 12:03 - 12:33
                    Started:     08:03
                    End:         16:33"
                    ),
                    format!("{}", status)
                );
            }
        }

        #[test]
        fn status_daily_temp_end() {
            logger();
            let local = chrono::Local::now().sub(Duration::minutes(35));
            let est_end = StatusTime::from(
                local.add(Duration::hours(8).add(Duration::minutes(30)))
            );
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: local,
                    },
                ].to_vec(),
                ..Default::default()
            };
            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 6 * 60,
                        minutes: 30,
                    },
                ].to_vec(),
                workperday: WorkPerDayInMinutes {
                    saturday: 8 * 60,
                    sunday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            let str = format!("{}", status);
            let lines = str.split('\n').collect::<Vec<&str>>();

            if ShouldColorize::from_env().should_colorize() {
                //assert worktime
                assert_eq!("Work time:   00:05 (\u{1b}[91m-07:55\u{1b}[0m)", lines[0]);
                //assert online
                assert_eq!("Online time: 00:35", lines[1]);
                //assert break
                assert_eq!("Break:       00:00 (\u{1b}[91m-00:30\u{1b}[0m)", lines[2]);

                assert!(
                    format!("{}", status).contains(
                        &format!("End:         \u{1b}[93m{} (est.)\u{1b}[0m", est_end)
                    ),
                    "Expected 'estimated end' to be: {}, but found: {}",
                    est_end,
                    status.est_end
                );
            } else {
                //assert worktime
                assert_eq!("Work time:   00:05 (-07:55)", lines[0]);
                //assert online
                assert_eq!("Online time: 00:35", lines[1]);
                //assert break
                assert_eq!("Break:       00:00 (-00:30)", lines[2]);

                assert!(
                    format!("{}", status).contains(&format!("End:         {}", est_end)),
                    "Expected 'estimated end' to be: {}', but found: {}",
                    est_end,
                    status.est_end
                );
            }
            assert!(!format!("{}", status).contains("Break taken"));
        }

        #[test]
        fn status_daily_short_day_without_break() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 22, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 16, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                ].to_vec(),
                workperday: WorkPerDayInMinutes {
                    wednesday: 6 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            if ShouldColorize::from_env().should_colorize() {
                assert_eq!(
                    indoc!(
                        "Work time:   03:54 (\u{1b}[91m-02:06\u{1b}[0m)
                        Online time: 03:54
                        Break:       00:00 (+00:00)

                        Started:     08:22
                        End:         \u{1b}[92m12:16\u{1b}[0m"
                    ),
                    format!("{}", status)
                );
            } else {
                assert_eq!(
                    indoc!(
                        "Work time:   03:54 (-02:06)
                    Online time: 03:54
                    Break:       00:00 (+00:00)

                    Started:     08:22
                    End:         12:16"
                    ),
                    format!("{}", status)
                );
            }
        }

        #[test]
        fn status_daily_should_ignore_takeover() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 22, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 16, 0).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Takeover,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 46, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                ].to_vec(),
                workperday: WorkPerDayInMinutes {
                    wednesday: 6 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            if ShouldColorize::from_env().should_colorize() {
                assert_eq!(
                    indoc!(
                        "Work time:   03:54 (\u{1b}[91m-02:06\u{1b}[0m)
                        Online time: 03:54
                        Break:       00:00 (+00:00)

                        Started:     08:22
                        End:         \u{1b}[92m12:16\u{1b}[0m"
                    ),
                    format!("{}", status)
                );
            } else {
                assert_eq!(
                    indoc!(
                        "Work time:   03:54 (-02:06)
                    Online time: 03:54
                    Break:       00:00 (+00:00)

                    Started:     08:22
                    End:         12:16"
                    ),
                    format!("{}", status)
                );
            }
        }
    }

    mod builder {
        use super::*;

        #[test]
        fn should_set_connect() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 15, 6, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };
            let status = StatusDaily::builder()
                .data(data)
                .settings(Settings::default())
                .build()
                .unwrap();
            assert!(status.has_connect());
        }

        #[test]
        #[should_panic]
        fn no_connect() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };
            StatusDaily::builder().data(data).settings(Settings::default()).build().unwrap();
        }

        #[test]
        fn should_set_end() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 10, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 10, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 10, 10, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let status = StatusDaily::builder()
                .data(data)
                .settings(Settings::default())
                .build()
                .unwrap();

            assert!(status.end.is_some());
            assert_eq!(10, status.end.unwrap().duration.num_hours())
        }

        #[test]
        fn should_set_temporary_end() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 2,
                        status: Status::Connect,
                        time: DateTime::default(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let status = StatusDaily::builder()
                .data(data)
                .settings(Settings::default())
                .build()
                .unwrap();

            assert!(status.end.is_none());
            assert!(status.temp_end.is_some());
            assert!(status.temp_end.unwrap().duration.num_seconds().ge(&0));
        }

        #[test]
        fn should_calculate_break() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 10, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 40, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 45, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let status = StatusDaily::builder()
                .data(data)
                .settings(Settings::default())
                .build()
                .unwrap();

            let r#break = status.r#break.unwrap();
            assert_eq!(0, r#break.hours);
            assert_eq!(5, r#break.minutes);
        }

        #[test]
        fn should_calculate_break_between_mutliple_breaks() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 10, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 40, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 40, 5).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 45, 0).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 9, 40, 0).unwrap(),
                    },
                    Entry {
                        id: 6,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 9, 40, 55).unwrap(),
                    },
                    Entry {
                        id: 7,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 9, 55, 0).unwrap(),
                    },
                    Entry {
                        id: 8,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 10, 0, 55).unwrap(),
                    },
                    Entry {
                        id: 9,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 10, 0, 56).unwrap(),
                    },
                    Entry {
                        id: 10,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 10, 1, 56).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let status = StatusDaily::builder()
                .data(data)
                .settings(Settings::default())
                .build()
                .unwrap();

            let r#break = status.r#break.unwrap();
            assert_eq!(0, r#break.hours);
            assert_eq!(21, r#break.minutes);
        }

        #[test]
        fn should_calculate_online_time() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 15, 6, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let settings = Settings::default();
            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            assert_eq!(7, status.online.as_ref().unwrap().hours);
            assert_eq!(3, status.online.as_ref().unwrap().minutes);
        }

        #[test]
        fn should_calculate_est_end_with_break_fully_taken() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 45, 0).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 5, 45, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };
            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();
            let expected_end = Duration::hours(8).add(Duration::minutes(45));
            assert_eq!(expected_end, status.est_end.duration);
        }

        #[test]
        fn should_calculate_est_end_with_break_not_fully_taken() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 0, 00, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 15, 0).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 5, 45, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            let expected_end = Duration::hours(8).add(Duration::minutes(45));
            assert_eq!(expected_end, status.est_end.duration);
        }

        #[test]
        fn should_calculate_est_end_with_more_break_taken() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 0, 00, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 4, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 5, 45, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();
            let expected_end = Duration::hours(9);
            assert_eq!(expected_end, status.est_end.duration);
        }

        #[test]
        fn should_calculate_est_end_with_no_break_taken() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 15, 0).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 5, 45, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    wednesday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();
            let expected_end = Duration::hours(8).add(Duration::minutes(45));
            assert_eq!(expected_end, status.est_end.duration);
        }

        #[test]
        fn should_calculate_est_end_with_no_break_taken_short_day_and_threshold() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 20, 0).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 5, 45, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    wednesday: 6 * 60,
                    ..Default::default()
                },
                threshold_limits: 5,
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();
            let expected_end = Duration::hours(6);
            assert_eq!(
                expected_end,
                status.est_end.duration,
                "expected end at 6:00 but got {}",
                status.est_end
            );
        }

        #[test]
        fn should_calculate_est_end_with_no_break_taken_odd_day() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Disconnect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 0, 1).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 3, 20, 0).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 5, 45, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    wednesday: 7 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();
            let expected_end = Duration::hours(7).add(Duration::minutes(30));
            assert_eq!(expected_end, status.est_end.duration);
        }

        #[test]
        fn should_calculate_no_break_on_short_day() {
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 22, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 16, 0).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };

            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                ].to_vec(),
                workperday: WorkPerDayInMinutes {
                    wednesday: 6 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();
            let expected_end = Duration::hours(14).add(Duration::minutes(22));
            assert_eq!(
                expected_end,
                status.est_end.duration,
                "expected to end at 14:22 but got {}",
                status.est_end
            );
            assert_eq!(
                Duration::minutes(0),
                status.exp_break.to_owned().unwrap().duration,
                "expected break to be 00:00 but got {}",
                status.exp_break.unwrap()
            );
        }
    }

    mod logic {
        use super::*;

        #[test]
        fn should_calculate_overtime() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 8, 55, 46).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 8, 56, 15).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 12, 25, 57).unwrap(),
                    },
                    Entry {
                        id: 4,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 12, 26, 46).unwrap(),
                    },
                    Entry {
                        id: 5,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 12, 28, 7).unwrap(),
                    },
                    Entry {
                        id: 6,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 12, 58, 7).unwrap(),
                    },
                    Entry {
                        id: 7,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 17, 0, 7).unwrap(),
                    },
                    Entry {
                        id: 8,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 17, 15, 7).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 18, 27, 40).unwrap(),
                    },
                ].to_vec(),
                ..Default::default()
            };
            let settings = Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 30,
                    },
                ].to_vec(),
                workperday: WorkPerDayInMinutes {
                    monday: 510,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            assert_eq!(
                Duration::minutes(16).add(Duration::seconds(5)),
                status.overtime.duration,
                "expected 0:16 overtime but was {}",
                status.overtime
            );
        }

        #[test]
        fn should_calculate_negative_overtime() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 8, 22, 11).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 7, 12, 16, 32).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    monday: 360,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);

            assert_eq!(
                Duration::hours(-2).sub(Duration::minutes(21).sub(Duration::seconds(21))),
                status.overtime.duration,
                "expected -2:21 overtime but was {}",
                status.overtime
            );
        }

        #[test]
        fn should_calculate_worktime_and_expected_break_with_only_30_minutes_break_taken() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 30, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 17, 0, 0).unwrap(),
                    },
                ].to_vec(),
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

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);
            assert_eq!(
                Duration::minutes(45),
                status.exp_break.as_ref().unwrap().duration,
                "expected 45 minutes break but was {}",
                status.exp_break.as_ref().unwrap()
            );
            assert_eq!(
                Duration::hours(8).add(Duration::minutes(15)),
                status.worktime.duration,
                "expected 8:15 working time but was {}",
                status.worktime
            )
        }

        #[test]
        fn should_calculate_worktime_and_expected_break_with_4_hours_break_taken() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 16, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 21, 0, 0).unwrap(),
                    },
                ].to_vec(),
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

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);
            assert_eq!(
                Duration::minutes(60),
                status.exp_break.as_ref().unwrap().duration,
                "expected 1:00 break but was {}",
                status.exp_break.as_ref().unwrap()
            );
            assert_eq!(
                Duration::hours(9),
                status.worktime.duration,
                "expected 9:00 working time but was {}",
                status.worktime
            )
        }

        #[test]
        fn should_calculate_worktime_and_expected_break_with_4_hours_break_taken_and_over_10_hours_worktime() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 12, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 16, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 3,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 23, 0, 0).unwrap(),
                    },
                ].to_vec(),
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

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);
            assert_eq!(
                Duration::hours(1),
                status.exp_break.as_ref().unwrap().duration,
                "expected 1:00 hour break but was {}",
                status.exp_break.as_ref().unwrap()
            );
            assert_eq!(
                Duration::hours(11),
                status.worktime.duration,
                "expected 11:00 working time but was {}",
                status.worktime
            )
        }

        #[test]
        fn should_calculate_worktime_and_no_expected_break_because_less_than_6_hours_worktime() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 4, 8, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 4, 14, 0, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    friday: 6 * 60,
                    ..Default::default()
                },
                threshold_limits: 5,
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);
            assert_eq!(
                Duration::minutes(0),
                status.exp_break.as_ref().unwrap().duration,
                "expected 0:00 hour break but was {}",
                status.exp_break.as_ref().unwrap()
            );
            assert_eq!(
                Duration::hours(6),
                status.worktime.duration,
                "expected 6:00 working time but was {}",
                status.worktime
            )
        }

        #[test]
        fn expected_break_not_displayed_correctly_in_status() {
            logger();
            let data = TimeData {
                entries: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap(),
                    },
                    Entry {
                        id: 2,
                        status: Status::End,
                        time: Local.with_ymd_and_hms(2022, 2, 2, 14, 0, 0).unwrap(),
                    },
                ].to_vec(),
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
                workperday: WorkPerDayInMinutes {
                    friday: 6 * 60,
                    ..Default::default()
                },
                threshold_limits: 5,
                ..Default::default()
            };

            let status = StatusDaily::builder().data(data).settings(settings).build().unwrap();

            log::debug!("{}", status);
            assert_eq!(
                Duration::minutes(45),
                status.exp_break.as_ref().unwrap().duration,
                "expected 0:45 hour break but was {}",
                status.exp_break.as_ref().unwrap()
            );
        }
    }
}
