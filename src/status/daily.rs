use std::ops::{ Add, Mul };

use chrono::{ DateTime, Duration, FixedOffset };
use colored::Colorize;
use prettytable::{ format, Table };

use crate::{ Entry, Settings, Status, StatusDisplay, StatusTime, Timesheet, TrackerError };

/// The status of a day, calculated from its timesheet.
pub struct Daily<'a> {
    timesheet: &'a Timesheet,
    now: DateTime<FixedOffset>,
    start: StatusTime,
    end: Option<StatusTime>,
    online: StatusTime,
    r#break: StatusTime,
    expected_break: StatusTime,
    estimated_end: DateTime<FixedOffset>,
    worktime: StatusTime,
    overtime: StatusTime,
}

impl<'a> Daily<'a> {
    /// Status of `timesheet` at `now`, which ends a day that is still running.
    pub fn new(
        timesheet: &'a Timesheet,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<Self, TrackerError> {
        if timesheet.data.is_empty() {
            return Err(TrackerError::StatusError {
                message: "data not added to status".to_owned(),
            });
        }
        let start = timesheet.start().ok_or_else(|| TrackerError::StatusError {
            message: "no initial connect found in data".to_owned(),
        })?;

        Ok(Daily {
            timesheet,
            now,
            start,
            end: timesheet.end(),
            online: timesheet.online(now)?,
            r#break: timesheet.break_taken(),
            expected_break: timesheet.expected_break(settings, now)?,
            estimated_end: timesheet.estimated_end(settings, now)?,
            worktime: timesheet.worktime(settings, now)?,
            overtime: timesheet.overtime(settings, now)?,
        })
    }

    /// Work and break ranges of the day, each starting at a connect or break entry and ending
    /// at the next entry. The last range has no end while it is still running.
    fn ranges(&self) -> Vec<(&Entry, Option<&Entry>)> {
        let entries = &self.timesheet.data;
        entries
            .iter()
            .enumerate()
            .filter(|(_, e)| matches!(e.status, Status::Connect | Status::Break))
            .map(|(i, e)| (e, entries.get(i + 1)))
            .collect()
    }
}

impl std::fmt::Display for Daily<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let zero_dr = Duration::minutes(0);
        let remaining = self.overtime.to_owned();

        let rm_fmt = match remaining.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Greater => format!("+{}", remaining).bright_green(),
            std::cmp::Ordering::Equal => format!("+{}", remaining).normal(),
            std::cmp::Ordering::Less => format!("-{}", remaining.mul(-1)).bright_red(),
        };

        let break_diff = self.expected_break.to_owned() - self.r#break.to_owned();

        let bk_fmt = match break_diff.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Less => format!("+{}", break_diff.mul(-1)).bright_yellow(),
            std::cmp::Ordering::Equal => format!("+{}", break_diff).normal(),
            std::cmp::Ordering::Greater => format!("-{}", break_diff).bright_red(),
        };

        let mut fmt_break_report = "".to_owned();

        let end_fmt = if let Some(end) = &self.end {
            if let Some(first_break) = self.timesheet.first_break() {
                fmt_break_report = format!(
                    "\n{:width$}{} - {}",
                    "Break taken:",
                    StatusTime::from(first_break),
                    StatusTime::from(first_break.add(self.r#break.duration)),
                    width = 13
                );
            }

            format!("{}", end).bright_green()
        } else if self.now >= self.estimated_end {
            format!("{}", StatusTime::from(self.now)).bright_green()
        } else {
            self.estimated_end.format("%H:%M (est.)").to_string().bright_yellow()
        };

        let line1 = format!("{:width$}{} ({})", "Work time:", self.worktime, rm_fmt, width = 13);
        let line2 = format!("{:width$}{}", "Online time:", self.online, width = 13);
        let notes: Vec<&str> = self.timesheet.data
            .iter()
            .filter_map(|e| e.note.as_deref())
            .collect();
        let activities = if notes.is_empty() {
            "".to_owned()
        } else {
            format!("\n{:width$}{}", "Activities:", notes.join(", "), width = 13)
        };
        let line3 = format!(
            "{:width$}{} ({}){}",
            "Break:",
            self.r#break,
            bk_fmt,
            activities,
            width = 13
        );
        let line4 = fmt_break_report;
        let line5 = format!("{:width$}{}", "Started:", self.start, width = 13);
        let line6 = format!("{:width$}{}", "End:", end_fmt, width = 13);
        write!(f, "{}\n{}\n{}\n{}\n{}\n{}", line1, line2, line3, line4, line5, line6)
    }
}

impl StatusDisplay for Daily<'_> {
    fn table(&self) -> String {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_COLSEP);
        table.set_titles(row!["Start", "End", "Duration", "Type", "Note"]);

        for (start, end) in self.ranges() {
            let end_time = end.map_or(self.now, |e| e.time);
            let kind = if start.status == Status::Break { "Break" } else { "Work" };
            table.add_row(
                row![
                    start.time.format("%H:%M"),
                    end.map_or("now".to_owned(), |e| e.time.format("%H:%M").to_string()),
                    StatusTime::from(end_time - start.time),
                    kind,
                    start.note.as_deref().unwrap_or_default()
                ]
            );
        }

        table.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::ops::{ Add, Sub };

    use chrono::{ DateTime, Duration, FixedOffset, TimeZone, Utc };

    use crate::{
        BreakLimit,
        Daily,
        Entry,
        Settings,
        Status,
        StatusDisplay,
        StatusTime,
        Timesheet,
        WorkPerDay,
    };

    use indoc::indoc;

    use colored::control::ShouldColorize;

    fn logger() {
        // std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// Clock for the tests, past the start of the days and before their estimated end.
    fn now() -> DateTime<FixedOffset> {
        Utc.with_ymd_and_hms(2022, 2, 2, 10, 0, 0).unwrap().fixed_offset()
    }

    mod new {
        use super::*;

        #[test]
        fn should_reject_empty_timesheet() {
            let timesheet = Timesheet::default();
            let err = Daily::new(&timesheet, &Settings::default(), now()).err().unwrap();

            assert_eq!("status error: data not added to status", err.to_string());
        }

        #[test]
        fn should_reject_timesheet_without_connect() {
            let timesheet = Timesheet {
                data: vec![Entry { status: Status::End, time: now(), note: None }],
                ..Default::default()
            };
            let err = Daily::new(&timesheet, &Settings::default(), now()).err().unwrap();

            assert_eq!("status error: no initial connect found in data", err.to_string());
        }
    }

    mod ranges {
        use super::*;

        fn entry(status: Status, hour: u32, note: Option<&str>) -> Entry {
            Entry {
                status,
                time: Utc.with_ymd_and_hms(2022, 2, 2, hour, 0, 0).unwrap().fixed_offset(),
                note: note.map(str::to_owned),
            }
        }

        fn timesheet(data: Vec<Entry>) -> Timesheet {
            Timesheet { data, ..Default::default() }
        }

        #[test]
        fn should_pair_ranges_with_notes() {
            let t = timesheet(
                vec![
                    entry(Status::Connect, 8, Some("standup")),
                    entry(Status::Break, 12, Some("lunch")),
                    entry(Status::Connect, 13, None),
                    entry(Status::End, 17, None)
                ]
            );
            let s = Daily::new(&t, &Settings::default(), now()).unwrap();
            let ranges: Vec<_> = s
                .ranges()
                .iter()
                .map(|(start, end)| (start.status.clone(), end.map(|e| e.status.clone()), start.note.clone()))
                .collect();

            assert_eq!(
                vec![
                    (Status::Connect, Some(Status::Break), Some("standup".to_owned())),
                    (Status::Break, Some(Status::Connect), Some("lunch".to_owned())),
                    (Status::Connect, Some(Status::End), None)
                ],
                ranges
            );
        }

        #[test]
        fn should_leave_running_range_open() {
            let t = timesheet(
                vec![entry(Status::Connect, 8, None), entry(Status::Break, 12, Some("lunch"))]
            );
            let s = Daily::new(&t, &Settings::default(), now()).unwrap();
            let ranges = s.ranges();

            assert_eq!(2, ranges.len());
            assert_eq!(Status::Break, ranges[1].0.status);
            assert!(ranges[1].1.is_none());
        }
    }

    mod table {
        use super::*;

        /// The words of each row, without the header and the separator lines.
        fn rows(table: &str) -> Vec<Vec<&str>> {
            table
                .lines()
                .filter(|l| !l.starts_with('-') && !l.starts_with('='))
                .skip(1)
                .map(|l| l.split_whitespace().collect())
                .collect()
        }

        #[test]
        fn should_show_a_row_for_each_range() {
            let at = |hour, minute| Utc.with_ymd_and_hms(2022, 2, 2, hour, minute, 0).unwrap().fixed_offset();
            let t = Timesheet {
                data: vec![
                    Entry { status: Status::Connect, time: at(8, 0), note: Some("standup".to_owned()) },
                    Entry { status: Status::Break, time: at(9, 30), note: Some("lunch".to_owned()) },
                    Entry { status: Status::Connect, time: at(9, 45), note: None },
                    Entry { status: Status::End, time: at(17, 0), note: None }
                ],
                ..Default::default()
            };
            let table = Daily::new(&t, &Settings::default(), now()).unwrap().table();

            assert!(table.contains("Start") && table.contains("Duration"), "{table}");
            assert_eq!(
                vec![
                    vec!["08:00", "09:30", "01:30", "Work", "standup"],
                    vec!["09:30", "09:45", "00:15", "Break", "lunch"],
                    vec!["09:45", "17:00", "07:15", "Work"]
                ],
                rows(&table)
            );
        }

        #[test]
        fn should_end_a_running_range_at_now() {
            let t = Timesheet {
                data: vec![
                    Entry {
                        status: Status::Connect,
                        time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 0, 0).unwrap().fixed_offset(),
                        note: None,
                    }
                ],
                ..Default::default()
            };
            let table = Daily::new(&t, &Settings::default(), now()).unwrap().table();

            assert_eq!(vec![vec!["08:00", "now", "02:00", "Work"]], rows(&table));
        }
    }

    mod display {
        use chrono::Local;


        use super::*;

        #[test]
        fn daily_with_remaing_worktime_and_break() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 3, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 23, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 14, 45, 0)
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
                ].to_vec(),
                ..Default::default()
            };

            let status = Daily::new(&data, &settings, now()).unwrap();

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
        fn daily_with_activities() {
            logger();
            let at = |hour, minute| Utc.with_ymd_and_hms(2022, 2, 2, hour, minute, 0).unwrap().fixed_offset();
            let data = Timesheet {
                data: vec![
                        Entry { status: Status::Connect, time: at(8, 0), note: Some("standup".to_owned()) },
                        Entry { status: Status::Break, time: at(12, 0), note: Some("lunch".to_owned()) },
                        Entry { status: Status::Connect, time: at(12, 30), note: None },
                        Entry { status: Status::Connect, time: at(13, 0), note: Some("review".to_owned()) },
                        Entry { status: Status::End, time: at(16, 30), note: None }
                    ],
                ..Default::default()
            };

            let status = Daily::new(&data, &Settings::default(), now()).unwrap();
            let out = format!("{}", status);

            let lines: Vec<&str> = out.lines().collect();
            assert!(lines[2].starts_with("Break:"));
            assert_eq!("Activities:  standup, lunch, review", lines[3]);
        }

        #[test]
        fn daily_without_notes_has_no_activities() {
            logger();
            let at = |hour, minute| Utc.with_ymd_and_hms(2022, 2, 2, hour, minute, 0).unwrap().fixed_offset();
            let data = Timesheet {
                data: vec![
                        Entry { status: Status::Connect, time: at(8, 0), note: None },
                        Entry { status: Status::Break, time: at(12, 0), note: None },
                        Entry { status: Status::Connect, time: at(12, 30), note: None },
                        Entry { status: Status::End, time: at(16, 30), note: None }
                    ],
                ..Default::default()
            };

            let status = Daily::new(&data, &Settings::default(), now()).unwrap();
            let out = format!("{}", status);

            assert!(!out.contains("Activities"));
            assert!(out.lines().nth(3).unwrap().is_empty());
        }

        #[test]
        fn daily_with_overtime_and_more_break() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 3, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 43, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 17, 45, 0)
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
                ].to_vec(),
                ..Default::default()
            };
            let status = Daily::new(&data, &settings, now()).unwrap();

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
        fn daily_on_point() {
            logger();
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap().fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Break,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 3, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::Connect,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 33, 0)
                                    .unwrap()
                                    .fixed_offset(),
                                note: None,
                            },
                            Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 16, 33, 0)
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
                ].to_vec(),
                ..Default::default()
            };

            let status = Daily::new(&data, &settings, now()).unwrap();

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
        fn daily_temp_end() {
            logger();
            let local = now().with_timezone(&Local).sub(Duration::minutes(35));
            let est_end = StatusTime::from(
                local.add(Duration::hours(8).add(Duration::minutes(30)))
            );
            let data = Timesheet {
                data: {
                    [
                            Entry {
                                status: Status::Connect,
                                time: local.fixed_offset(),
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
                ].to_vec(),
                workperday: WorkPerDay {
                    saturday: 8 * 60,
                    sunday: 8 * 60,
                    ..Default::default()
                },
                ..Default::default()
            };

            let status = Daily::new(&data, &settings, now()).unwrap();

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
                    est_end
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
                    est_end
                );
            }
            assert!(!format!("{}", status).contains("Break taken"));
        }

        #[test]
        fn daily_short_day_without_break() {
            logger();
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

            let status = Daily::new(&data, &settings, now()).unwrap();

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
        fn daily_should_ignore_takeover() {
            logger();
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
                            Entry {
                                status: Status::Takeover,
                                time: Utc.with_ymd_and_hms(2022, 2, 2, 12, 46, 0)
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

            let status = Daily::new(&data, &settings, now()).unwrap();

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

    mod crossing_midnight {
        use super::*;

        /// A running day that started on Monday at 22:00, with a target of 8 hours.
        fn timesheet() -> Timesheet {
            Timesheet {
                data: vec![Entry {
                    status: Status::Connect,
                    time: Utc.with_ymd_and_hms(2022, 2, 7, 22, 0, 0).unwrap().fixed_offset(),
                    note: None,
                }],
                ..Default::default()
            }
        }

        fn out(now: DateTime<FixedOffset>) -> String {
            Daily::new(&timesheet(), &Settings::default(), now).unwrap().to_string()
        }

        #[test]
        fn should_show_the_estimated_end_after_midnight_until_it_is_reached() {
            let now = Utc.with_ymd_and_hms(2022, 2, 8, 1, 0, 0).unwrap().fixed_offset();
            let out = out(now);

            assert!(out.contains("06:00 (est.)"), "{out}");
            assert!(out.contains("Online time: 03:00"), "{out}");
        }

        #[test]
        fn should_show_now_once_the_estimated_end_is_reached() {
            let now = Utc.with_ymd_and_hms(2022, 2, 8, 7, 30, 0).unwrap().fixed_offset();
            let out = out(now);

            assert!(!out.contains("(est.)"), "{out}");
            assert!(out.contains("07:30"), "{out}");
            assert!(out.contains("Online time: 09:30"), "{out}");
        }

        #[test]
        fn should_show_the_times_of_the_entries_in_their_own_offsets() {
            let t = Timesheet {
                data: vec![
                    Entry {
                        status: Status::Connect,
                        time: FixedOffset::east_opt(3600)
                            .unwrap()
                            .with_ymd_and_hms(2022, 2, 7, 8, 0, 0)
                            .unwrap(),
                        note: None,
                    },
                    Entry {
                        status: Status::End,
                        time: FixedOffset::east_opt(-5 * 3600)
                            .unwrap()
                            .with_ymd_and_hms(2022, 2, 7, 17, 0, 0)
                            .unwrap(),
                        note: None,
                    }
                ],
                ..Default::default()
            };
            let daily = Daily::new(&t, &Settings::default(), now()).unwrap();
            let table = daily.table();

            // 08:00 in Berlin to 17:00 in New York.
            assert!(table.contains("08:00"), "{table}");
            assert!(table.contains("17:00"), "{table}");
            assert!(table.contains("15:00"), "{table}");
            assert!(daily.to_string().contains("Started:     08:00"), "{daily}");
        }
    }
}
