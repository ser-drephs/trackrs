use std::{ env, fmt::Display, ops::Mul };

use chrono::{ DateTime, Duration, Local };
use colored::Colorize;
use prettytable::{ format, Table };

use crate::{ Settings, StatusDaily, StatusTime, TimeData, TimeDataWeekly, TrackerError };

#[derive(Clone, Default, Debug)]
pub struct StatusWeekly {
    week: i8,
    total: StatusTime,
    overtime: StatusTime,
    decimal: f64,
    entries: Vec<(DateTime<Local>, StatusDaily)>,
}

impl StatusWeekly {
    pub fn builder() -> StatusWeeklyBuilder {
        StatusWeeklyBuilder {
            ..Default::default()
        }
    }

    pub fn format_table(&self) {
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
                format!("{:width$}", "Overtime")
            ]
        );

        fn status_unwrap(status: Option<StatusTime>) -> StatusTime {
            match status {
                Some(val) => val,
                None => StatusTime::default(),
            }
        }

        log::trace!("iterate over entries");

        self.entries.iter().for_each(|(date, status)| {
            table.add_row(
                row![
                    date.to_owned().format("%a %d %b"),
                    format!("{}", status_unwrap(status.start.to_owned())),
                    format!("{}", status_unwrap(status.end.to_owned())),
                    format!("{}", status_unwrap(status.r#break.to_owned())),
                    format!("{}", status.worktime),
                    format!("{}", status.overtime)
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
                self.overtime
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

        let mut entries: Vec<(DateTime<Local>, StatusDaily)> = Vec::new();

        let mut total = StatusTime::default();
        let mut overtime = StatusTime::default();

        data.entries.iter().for_each(|d: &TimeData| {
            log::trace!("processing: {:?}", d);
            let mut b = StatusDaily::builder();
            if !d.entries.is_empty() {
                let s = b.data(d.to_owned()).settings(settings.clone()).build().unwrap();
                log::info!("got {} working time and {} overtime", s.worktime, s.overtime);
                entries.append(&mut [(d.date.unwrap().to_owned(), s.to_owned())].to_vec());
                total += s.worktime;
                overtime += s.overtime;
            } else {
                let expected = self.settings
                    .as_ref()
                    .unwrap()
                    .workperday.from_date(d.date.unwrap());
                if expected >= &0 {
                    let exh = expected.to_owned() as i64;
                    overtime -= StatusTime::from(Duration::minutes(exh));
                    let missing_status = StatusDaily::builder().empty_with_overtime(
                        overtime.to_owned()
                    );
                    entries.append(&mut [(d.date.unwrap().to_owned(), missing_status)].to_vec());
                }
            }
        });

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

    use chrono::{ Local, TimeZone };

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

        fn get_entries(day: u8, end: u8, end_minutes: u8) -> Vec<Entry> {
            [
                Entry {
                    id: 1,
                    status: Status::Connect,
                    time: Local.with_ymd_and_hms(2022, 3, day.into(), 0, 0, 0).unwrap(),
                },
                Entry {
                    id: 2,
                    status: Status::Break,
                    time: Local.with_ymd_and_hms(2022, 3, day.into(), 4, 0, 0).unwrap(),
                },
                Entry {
                    id: 3,
                    status: Status::Connect,
                    time: Local.with_ymd_and_hms(2022, 3, day.into(), 4, 30, 0).unwrap(),
                },
                Entry {
                    id: 4,
                    status: Status::End,
                    time: Local.with_ymd_and_hms(
                        2022,
                        3,
                        day.into(),
                        end.into(),
                        end_minutes.into(),
                        0
                    ).unwrap(),
                },
            ].to_vec()
        }

        fn get_time_data(one_day_end: u8, one_day_minute_off: u8) -> Vec<TimeData> {
            [
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 7, 0, 0, 0).unwrap()),
                    entries: get_entries(7, 8, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 8, 0, 0, 0).unwrap()),
                    entries: get_entries(8, 8, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 9, 0, 0, 0).unwrap()),
                    entries: get_entries(9, one_day_end, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 10, 0, 0, 0).unwrap()),
                    entries: get_entries(10, 8, one_day_minute_off),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 11, 0, 0, 0).unwrap()),
                    entries: get_entries(11, 8, 30),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 12, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 13, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
            ].to_vec()
        }

        #[test]
        fn should_calculate_overtime() -> Result<(), TrackerError> {
            logger();
            let time_data = get_time_data(10, 42);
            let settings = get_settings();
            let time_data_weekly = TimeDataWeekly {
                entries: time_data,
                week: 10,
            };

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
            let time_data_weekly = TimeDataWeekly {
                entries: time_data,
                week: 10,
            };

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
            let time_data_weekly = TimeDataWeekly {
                entries: time_data,
                week: 10,
            };

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
                    date: Some(Local.with_ymd_and_hms(2022, 3, 7, 0, 0, 0).unwrap()),
                    entries: get_entries(7, 8, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 8, 0, 0, 0).unwrap()),
                    entries: get_entries(8, 8, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 9, 0, 0, 0).unwrap()),
                    entries: get_entries(9, 8, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 10, 0, 0, 0).unwrap()),
                    entries: get_entries(10, 10, 45),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 11, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 12, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
                TimeData {
                    date: Some(Local.with_ymd_and_hms(2022, 3, 13, 0, 0, 0).unwrap()),
                    ..Default::default()
                },
            ].to_vec();
            let settings = get_settings();
            let time_data_weekly = TimeDataWeekly {
                entries: time_data,
                week: 10,
            };

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
            Ok(())
        }

        #[test]
        fn should_not_crash_on_format_table() -> Result<(), TrackerError> {
            logger();
            test_env();
            let time_data = get_time_data(10, 42);
            let settings = get_settings();
            let time_data_weekly = TimeDataWeekly {
                entries: time_data,
                week: 10,
            };

            let mut b = StatusWeekly::builder();
            let s = b.data(time_data_weekly).settings(settings).build()?;
            s.format_table();
            Ok(())
        }
    }
}
