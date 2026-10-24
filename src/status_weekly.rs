use std::{ env, fmt::Display, ops::Mul };

use chrono::{ Duration, NaiveDate, NaiveTime };
use colored::Colorize;
use prettytable::{ format, Table };

use crate::{ Settings, StatusDaily, StatusTime, TimeDataWeekly, TrackerError };

#[derive(Clone, Default, Debug)]
pub struct StatusWeekly {
    pub(crate) week: i16,
    pub(crate) total: StatusTime,
    pub(crate) overtime: StatusTime,
    decimal: f64,
    entries: Vec<(NaiveDate, StatusDaily)>,
}

impl StatusWeekly {
    pub fn builder() -> StatusWeeklyBuilder {
        StatusWeeklyBuilder {
            ..Default::default()
        }
    }

    /// First day of the week, or of its days that were loaded.
    pub fn start(&self) -> Option<NaiveDate> {
        self.entries.first().map(|(d, _)| *d)
    }

    /// Print the table; `carry` is the overtime accumulated before the week.
    pub fn format_table(&self, carry: Option<StatusTime>) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_COLSEP);

        log::trace!("generating status table");

        let width = 7;

        log::trace!("set table titels");

        table.set_titles(
            row![
                format!("{:width$}", "Date"),
                format!("{:width$}", "Start"),
                format!("{:width$}", "End"),
                format!("{:width$}", "Break"),
                format!("{:width$}", "Worktime"),
                format!("{:width$}", "Overtime"),
                format!("{:width$}", "Cumulated")
            ]
        );

        fn status_unwrap(status: Option<StatusTime>) -> StatusTime {
            match status {
                Some(val) => val,
                None => StatusTime::default(),
            }
        }

        log::trace!("iterate over entries");

        if let Some(carry) = &carry {
            table.add_row(row!["Carried over", "", "", "", "", "", carry]);
        }
        let cumulated = StatusTime::cumulated(
            carry.unwrap_or_default(),
            self.entries.iter().map(|(_, s)| &s.overtime)
        );
        self.entries.iter().zip(cumulated).for_each(|((date, status), cumulated)| {
            table.add_row(
                row![
                    date.to_owned().format("%a %d %b"),
                    format!("{}", status_unwrap(status.start.to_owned())),
                    format!("{}", status_unwrap(status.end.to_owned())),
                    format!("{}", status_unwrap(status.r#break.to_owned())),
                    format!("{}", status.worktime),
                    format!("{}", status.overtime),
                    format!("{}", cumulated)
                ]
            );
        });

        table.add_empty_row();

        log::trace!("add summary row");

        table.add_row(
            row![
                format!("Total week {}", self.week),
                "",
                "",
                self.decimal,
                self.total,
                self.overtime,
                ""
            ]
        );

        if env::var("RUST_TEST").is_err() {
            log::trace!("print table to std");
            table.printstd();
        }
    }
}

#[derive(Default)]
pub struct StatusWeeklyBuilder {
    settings: Option<Settings>,
    data: Option<TimeDataWeekly>,
}

impl StatusWeeklyBuilder {
    pub fn data(&mut self, data: TimeDataWeekly) -> &mut Self {
        self.data = Some(data);
        self
    }

    pub fn settings(&mut self, settings: Settings) -> &mut Self {
        self.settings = Some(settings);
        self
    }

    pub fn build(&self) -> Result<StatusWeekly, TrackerError> {
        let settings = match self.settings.to_owned() {
            Some(s) => s,
            None => {
                return Err(TrackerError::StatusWeeklyError {
                    message: "settings not defined".to_owned(),
                });
            }
        };

        let data = match self.data.to_owned() {
            Some(d) => d,
            None => {
                return Err(TrackerError::StatusWeeklyError {
                    message: "data not defined".to_owned(),
                });
            }
        };

        let mut entries: Vec<(NaiveDate, StatusDaily)> = Vec::new();

        let mut total = StatusTime::default();
        let mut overtime = StatusTime::default();

        for date in &data.dates {
            log::trace!("processing: {}", date);
            match StatusDaily::load(&data.folder, &settings, *date)? {
                Some(s) => {
                    log::info!("got {} working time and {} overtime", s.worktime, s.overtime);
                    total += s.worktime.to_owned();
                    overtime += s.overtime.to_owned();
                    entries.push((*date, s));
                }
                None => {
                    let expected = settings.workperday.from_date(
                        date.and_time(NaiveTime::MIN).and_utc()
                    );
                    // A day without entries misses its whole target worktime.
                    let day_overtime = StatusTime::from(Duration::minutes(-(*expected as i64)));
                    overtime += day_overtime.to_owned();
                    let missing_status = StatusDaily::builder().empty_with_overtime(day_overtime);
                    entries.push((*date, missing_status));
                }
            }
        }

        log::info!("totally {} working time and {} overtime", total, overtime);

        // let i_32: i32 = total.duration.num_minutes().try_into()?;
        let decimal: f64 = self.datetime_to_decimal(&total);
        let week = data.week.to_owned();
        let sw = StatusWeekly {
            week,
            total,
            overtime,
            decimal: decimal.to_owned(),
            entries,
        };

        Ok(sw)
    }

    fn datetime_to_decimal(&self, total: &StatusTime) -> f64 {
        let hours = total.hours as f64; //.try_into().unwrap();
        let minutes = total.minutes as f64;
        let md = minutes * (1.0 / 60.0);
        hours + md
    }
}

impl Display for StatusWeekly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let zero_dr = Duration::minutes(0);
        let overtime = self.overtime.clone();

        let width = 10;

        let line1 = format!(
            " {:width$} | {:width$} | {:width$} | {:width$}",
            "Week",
            "Work time",
            "Overtime",
            "Decimal"
        );
        let line2 = format!(" {0:->width$} | {0:->width$} | {0:->width$} | {0:->width$}", "");

        let ot_fmt = match overtime.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Less => format!("-{}", overtime.mul(-1)).bright_red(),
            std::cmp::Ordering::Equal => format!("{}", overtime).normal(),
            std::cmp::Ordering::Greater => format!("+{}", overtime).bright_yellow(),
        };

        let dc_fmt = format!("{:.2}", self.decimal);
        let t_fmt = format!("{}", self.total);

        let line3 = format!(
            " {0:width$} | {1: >width$} | {2: >width$} | {3: >width$}",
            self.week,
            t_fmt,
            ot_fmt,
            dc_fmt
        );
        write!(f, "{}\n{}\n{}\n", line1, line2, line3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::ops::Add;

    use chrono::TimeZone;

    use crate::{ BreakLimit, Entry, Status };

    fn logger() {
        // std::env::set_var("RUST_LOG", "trace");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    fn test_env() {
        env::set_var("RUST_TEST", "true");
    }

    mod format {
        use colored::control::ShouldColorize;

        use super::*;

        #[test]
        fn status_overtime() {
            logger();
            let s = StatusWeekly {
                week: 23,
                total: StatusTime::from(Duration::hours(41).add(Duration::minutes(22))),
                overtime: StatusTime::from(Duration::minutes(82)),
                decimal: 42.5,
                entries: Vec::new(),
            };
            log::debug!("{}", s);

            if ShouldColorize::from_env().should_colorize() {
                assert_eq!(
                    " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      41:22 | \u{1b}[93m    +01:22\u{1b}[0m |      42.50\n",
                    format!("{}", s)
                );
            } else {
                assert_eq!(
                    " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      41:22 |     +01:22 |      42.50\n",
                    format!("{}", s)
                );
            }
        }

        #[test]
        fn status_on_point() {
            logger();
            let s = StatusWeekly {
                week: 23,
                total: StatusTime::from(Duration::hours(40)),
                overtime: StatusTime::from(Duration::minutes(0)),
                decimal: 40.0,
                entries: Vec::new(),
            };
            log::debug!("{}", s);

            assert_eq!(
                " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      40:00 |      00:00 |      40.00\n",
                format!("{}", s)
            );
        }

        #[test]
        fn status_less() {
            logger();
            let s = StatusWeekly {
                week: 23,
                total: StatusTime::from(Duration::hours(38).add(Duration::minutes(22))),
                overtime: StatusTime::from(Duration::minutes(-98)),
                decimal: 38.3,
                entries: Vec::new(),
            };
            log::debug!("{}", s);

            if ShouldColorize::from_env().should_colorize() {
                assert_eq!(
                    " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      38:22 | \u{1b}[91m    -01:38\u{1b}[0m |      38.30\n",
                    format!("{}", s)
                );
            } else {
                assert_eq!(
                    " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      38:22 |     -01:38 |      38.30\n",
                    format!("{}", s)
                );
            }
        }
    }

    mod builder {
        use chrono::Utc;

        use crate::{ Entries, TimeData };

        use super::*;

        fn get_settings() -> Settings {
            Settings {
                limits: [
                    BreakLimit {
                        start: 8 * 60,
                        minutes: 45,
                    },
                ].to_vec(),
                ..Default::default()
            }
        }

        fn get_entries(day: u8, end: u8, end_minutes: u8) -> Entries {
            Entries {
                data: [
                    Entry {
                        id: 1,
                        status: Status::Connect,
                        time: Utc.with_ymd_and_hms(2022, 3, day.into(), 0, 0, 0).unwrap(),
                        note: None,
                    },
                    Entry {
                        id: 2,
                        status: Status::Break,
                        time: Utc.with_ymd_and_hms(2022, 3, day.into(), 4, 0, 0).unwrap(),
                        note: None,
                    },
                    Entry {
                        id: 3,
                        status: Status::Connect,
                        time: Utc.with_ymd_and_hms(2022, 3, day.into(), 4, 30, 0).unwrap(),
                        note: None,
                    },
                    Entry {
                        id: 4,
                        status: Status::End,
                        time: Utc.with_ymd_and_hms(
                            2022,
                            3,
                            day.into(),
                            end.into(),
                            end_minutes.into(),
                            0
                        ).unwrap(),
                        note: None,
                    },
                ].to_vec(),
                ..Default::default()
            }
        }

        /// Write the days with entries to `folder` and return the week of all their dates.
        fn weekly(days: Vec<TimeData>, folder: &std::path::Path) -> Result<TimeDataWeekly, TrackerError> {
            for day in &days {
                if !day.entries.data.is_empty() {
                    let mut t = TimeData::builder()
                        .folder(folder.to_path_buf().into())
                        .date(day.date.unwrap())
                        .build()?;
                    t.entries = day.entries.to_owned();
                    t.write_to_file()?;
                }
            }
            Ok(TimeDataWeekly {
                folder: folder.to_path_buf().into(),
                dates: days.iter().map(|d| d.date.unwrap().date_naive()).collect(),
                week: 10,
            })
        }

        fn get_time_data(one_day_end: u8, one_day_minute_off: u8) -> Vec<TimeData> {
            [
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 7, 0, 0, 0).unwrap()),
                    entries: get_entries(7, 8, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 8, 0, 0, 0).unwrap()),
                    entries: get_entries(8, 8, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 9, 0, 0, 0).unwrap()),
                    entries: get_entries(9, one_day_end, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 10, 0, 0, 0).unwrap()),
                    entries: get_entries(10, 8, one_day_minute_off),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 11, 0, 0, 0).unwrap()),
                    entries: get_entries(11, 8, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 12, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 13, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
            ].to_vec()
        }

        #[test]
        fn should_calculate_overtime() -> Result<(), TrackerError> {
            logger();
            let time_data = get_time_data(10, 42);
            let settings = get_settings();
            let temp_dir = tempfile::tempdir()?;
            let time_data_weekly = weekly(time_data, temp_dir.path())?;

            let mut b = StatusWeekly::builder();
            let s = b.data(time_data_weekly).settings(settings).build()?;

            log::debug!("{}", s);

            let week = 10;
            let total = StatusTime::from(Duration::hours(40).add(Duration::minutes(57)));
            let overtime = StatusTime::from(Duration::minutes(57));
            let decimal = 40.95;

            assert_eq!(week, s.week);
            assert_eq!(total, s.total);
            assert_eq!(overtime, s.overtime);
            assert_eq!(decimal, s.decimal);
            Ok(())
        }

        #[test]
        fn should_calculate_on_point() -> Result<(), TrackerError> {
            logger();
            let time_data = get_time_data(9, 45);
            let settings = get_settings();
            let temp_dir = tempfile::tempdir()?;
            let time_data_weekly = weekly(time_data, temp_dir.path())?;

            let mut b = StatusWeekly::builder();
            let s = b.data(time_data_weekly).settings(settings).build()?;

            log::debug!("{}", s);

            let week = 10;
            let total = StatusTime::from(Duration::hours(40));
            let overtime = StatusTime::from(Duration::minutes(0));
            let decimal = 40.0;

            assert_eq!(week, s.week);
            assert_eq!(total, s.total);
            assert_eq!(overtime, s.overtime);
            assert_eq!(decimal, s.decimal);
            Ok(())
        }

        #[test]
        fn should_calculate_less() -> Result<(), TrackerError> {
            logger();
            let time_data = get_time_data(6, 17);
            let settings = get_settings();
            let temp_dir = tempfile::tempdir()?;
            let time_data_weekly = weekly(time_data, temp_dir.path())?;

            let mut b = StatusWeekly::builder();
            let s = b.data(time_data_weekly).settings(settings).build()?;

            log::debug!("{}", s);

            let week = 10;
            let total = StatusTime::from(Duration::hours(36).add(Duration::minutes(32)));
            let overtime = StatusTime::from(Duration::hours(-3).add(Duration::minutes(-28)));
            let decimal = 36.53333333333333;

            assert_eq!(week, s.week, "number of week");
            assert_eq!(total, s.total, "total calculation");
            assert_eq!(overtime, s.overtime, "overtime calculation");
            assert_eq!(decimal, s.decimal, "decimal representation");
            Ok(())
        }

        #[test]
        fn should_calculate_with_missing_day() -> Result<(), TrackerError> {
            logger();
            let time_data = [
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 7, 0, 0, 0).unwrap()),
                    entries: get_entries(7, 8, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 8, 0, 0, 0).unwrap()),
                    entries: get_entries(8, 8, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 9, 0, 0, 0).unwrap()),
                    entries: get_entries(9, 8, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 10, 0, 0, 0).unwrap()),
                    entries: get_entries(10, 10, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 11, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 12, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Utc.with_ymd_and_hms(2022, 3, 13, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
            ].to_vec();
            let settings = get_settings();
            let temp_dir = tempfile::tempdir()?;
            let time_data_weekly = weekly(time_data, temp_dir.path())?;

            let mut b = StatusWeekly::builder();
            let s = b.data(time_data_weekly).settings(settings).build()?;

            log::debug!("{}", s);

            let week = 10;
            let total = StatusTime::from(Duration::hours(34));
            let overtime = StatusTime::from(Duration::hours(-6));
            let decimal = 34.0;

            assert_eq!(week, s.week);
            assert_eq!(total, s.total);
            assert_eq!(overtime, s.overtime);
            assert_eq!(decimal, s.decimal);

            // Each row shows its own day's overtime, not the running total of the week.
            let rows: Vec<String> = s.entries
                .iter()
                .map(|(_, d)| d.overtime.to_string())
                .collect();
            assert_eq!(vec!["00:00", "00:00", "00:00", "02:00", "-08:00", "00:00", "00:00"], rows);
            Ok(())
        }

        #[test]
        fn should_not_crash_on_format_table() -> Result<(), TrackerError> {
            logger();
            test_env();
            let time_data = get_time_data(10, 42);
            let settings = get_settings();
            let temp_dir = tempfile::tempdir()?;
            let time_data_weekly = weekly(time_data, temp_dir.path())?;

            let mut b = StatusWeekly::builder();
            let s = b.data(time_data_weekly).settings(settings).build()?;
            s.format_table(None);
            Ok(())
        }
    }
}
