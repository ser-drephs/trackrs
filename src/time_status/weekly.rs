use std::{fmt::Display, ops::Mul};

use chrono::Duration;
use colored::Colorize;

use crate::{Settings, StatusDaily, TimeStatus, TimeStatusDaily, TimeStatusWeekly, TrackerError};
use super::

#[derive(Clone, Default, Debug)]
pub struct TimeStatusWeekly {
    week: i8,
    total: TimeStatus,
    overtime: TimeStatus,
    decimal: f64,
}

impl TimeStatusWeekly {
    pub fn builder() -> WeeklyBuilder {
        WeeklyBuilder::default()
    }
}

impl Display for TimeStatusWeekly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let zero_dr = Duration::minutes(0);
        let overtime = self.overtime.clone();

        let width = 10;

        let line1 = format!(
            " {:width$} | {:width$} | {:width$} | {:width$}",
            "Week", "Work time", "Overtime", "Decimal",
        );
        let line2 = format!(
            " {0:->width$} | {0:->width$} | {0:->width$} | {0:->width$}",
            "",
        );

        let ot_fmt = match overtime.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Less => format!("-{}", overtime.mul(-1)).bright_red(),
            std::cmp::Ordering::Equal => format!("{}", overtime).normal(),
            std::cmp::Ordering::Greater => format!("+{}", overtime).bright_yellow(),
        };

        let dc_fmt = format!("{:.2}", self.decimal);
        let t_fmt = format!("{}", self.total);

        let line3 = format!(
            " {0:width$} | {1: >width$} | {2: >width$} | {3: >width$}",
            self.week, t_fmt, ot_fmt, dc_fmt,
        );
        write!(f, "{}\n{}\n{}\n", line1, line2, line3)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use std::ops::Add;

    use chrono::{Local, TimeZone};

    use crate::{Entry, Status};

    fn logger() {
        // std::env::set_var("RUST_LOG", "trace");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // mod format {

    //     use colored::control::ShouldColorize;

    //     use super::*;

    //     #[test]
    //     fn status_overtime() {
    //         logger();
    //         let s = TimeStatusWeekly {
    //             week: 23,
    //             total: TimeStatus::from(Duration::hours(41).add(Duration::minutes(22))),
    //             overtime: TimeStatus::from(Duration::minutes(82)),
    //             decimal: 42.5,
    //         };
    //         log::debug!("{}", s);

    //         if ShouldColorize::from_env().should_colorize() {
    //             assert_eq!(
    //             " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      41:22 | \u{1b}[93m    +01:22\u{1b}[0m |      42.50\n",
    //             format!("{}", s)
    //         );
    //         } else {
    //             assert_eq!(
    //             " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      41:22 |     +01:22 |      42.50\n",
    //             format!("{}", s)
    //         );
    //         }
    //     }

    //     #[test]
    //     fn status_on_point() {
    //         logger();
    //         let s = TimeStatusWeekly {
    //             week: 23,
    //             total: TimeStatus::from(Duration::hours(40)),
    //             overtime: TimeStatus::from(Duration::minutes(0)),
    //             decimal: 40.0,
    //         };
    //         log::debug!("{}", s);

    //         assert_eq!(
    //             " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      40:00 |      00:00 |      40.00\n",
    //             format!("{}", s)
    //         );
    //     }

    //     #[test]
    //     fn status_less() {
    //         logger();
    //         let s = TimeStatusWeekly {
    //             week: 23,
    //             total: TimeStatus::from(Duration::hours(38).add(Duration::minutes(22))),
    //             overtime: TimeStatus::from(Duration::minutes(-98)),
    //             decimal: 38.3,
    //         };
    //         log::debug!("{}", s);

    //         if ShouldColorize::from_env().should_colorize() {
    //             assert_eq!(
    //             " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      38:22 | \u{1b}[91m    -01:38\u{1b}[0m |      38.30\n",
    //             format!("{}", s)
    //         );
    //         } else {
    //             assert_eq!(
    //             " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      38:22 |     -01:38 |      38.30\n",
    //             format!("{}", s)
    //         );
    //         }
    //     }
    // }

    // mod builder { // todo: tests

    //     use crate::settings::BreakLimit;

    //     use super::*;

    //     fn get_settings() -> Settings {
    //         Settings {
    //             limits: [BreakLimit {
    //                 start: 8 * 60,
    //                 minutes: 45,
    //             }]
    //             .to_vec(),
    //             ..Default::default()
    //         }
    //     }

    //     fn get_entries(day: u8, end: u8, end_minutes: u8) -> Vec<Entry> {
    //         [
    //             Entry {
    //                 id: 1,
    //                 status: Status::Connect,
    //                 time: Local.ymd(2022, 3, day.into()).and_hms(0, 0, 0),
    //             },
    //             Entry {
    //                 id: 2,
    //                 status: Status::Break,
    //                 time: Local.ymd(2022, 3, day.into()).and_hms(4, 0, 0),
    //             },
    //             Entry {
    //                 id: 3,
    //                 status: Status::Connect,
    //                 time: Local.ymd(2022, 3, day.into()).and_hms(4, 30, 0),
    //             },
    //             Entry {
    //                 id: 4,
    //                 status: Status::End,
    //                 time: Local
    //                     .ymd(2022, 3, day.into())
    //                     .and_hms(end.into(), end_minutes.into(), 0),
    //             },
    //         ]
    //         .to_vec()
    //     }

    //     fn get_time_data(one_day_end: u8, one_day_minute_off: u8) -> Vec<TimeStatus> {
    //         [
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 7)),
    //                 entries: get_entries(7, 8, 30),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 8)),
    //                 entries: get_entries(8, 8, 30),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 9)),
    //                 entries: get_entries(9, one_day_end, 30),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 10)),
    //                 entries: get_entries(10, 8, one_day_minute_off),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 11)),
    //                 entries: get_entries(11, 8, 30),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 12)),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 13)),
    //                 ..Default::default()
    //             },
    //         ]
    //         .to_vec()
    //     }

    //     #[test]
    //     fn should_calculate_overtime() -> Result<(), TrackerError> {
    //         logger();
    //         let TimeStatus = get_time_data(10, 42);
    //         let settings = get_settings();
    //         let TimeStatus_weekly = TimeStatusWeekly {
    //             entries: TimeStatus,
    //             week: 10,
    //         };

    //         let mut b = TimeStatusWeekly::builder();
    //         let s = b.data(TimeStatus_weekly).settings(settings).build()?;

    //         log::debug!("{}", s);

    //         let week = 10;
    //         let total = TimeStatus::from(Duration::hours(40).add(Duration::minutes(57)));
    //         let overtime = TimeStatus::from(Duration::minutes(57));
    //         let decimal = 40.95;

    //         assert_eq!(week, s.week);
    //         assert_eq!(total, s.total);
    //         assert_eq!(overtime, s.overtime);
    //         assert_eq!(decimal, s.decimal);
    //         Ok(())
    //     }

    //     #[test]
    //     fn should_calculate_on_point() -> Result<(), TrackerError> {
    //         logger();
    //         let TimeStatus = get_time_data(9, 45);
    //         let settings = get_settings();
    //         let TimeStatus_weekly = TimeStatusWeekly {
    //             entries: TimeStatus,
    //             week: 10,
    //         };

    //         let mut b = TimeStatusWeekly::builder();
    //         let s = b.data(TimeStatus_weekly).settings(settings).build()?;

    //         log::debug!("{}", s);

    //         let week = 10;
    //         let total = TimeStatus::from(Duration::hours(40));
    //         let overtime = TimeStatus::from(Duration::minutes(0));
    //         let decimal = 40.0;

    //         assert_eq!(week, s.week);
    //         assert_eq!(total, s.total);
    //         assert_eq!(overtime, s.overtime);
    //         assert_eq!(decimal, s.decimal);
    //         Ok(())
    //     }

    //     #[test]
    //     fn should_calculate_less() -> Result<(), TrackerError> {
    //         logger();
    //         let TimeStatus = get_time_data(6, 17);
    //         let settings = get_settings();
    //         let TimeStatus_weekly = TimeStatusWeekly {
    //             entries: TimeStatus,
    //             week: 10,
    //         };

    //         let mut b = TimeStatusWeekly::builder();
    //         let s = b.data(TimeStatus_weekly).settings(settings).build()?;

    //         log::debug!("{}", s);

    //         let week = 10;
    //         let total = TimeStatus::from(Duration::hours(36).add(Duration::minutes(32)));
    //         let overtime = TimeStatus::from(Duration::hours(-3).add(Duration::minutes(-28)));
    //         let decimal = 36.53333333333333;

    //         assert_eq!(week, s.week, "number of week");
    //         assert_eq!(total, s.total, "total calculation");
    //         assert_eq!(overtime, s.overtime, "overtime calculation");
    //         assert_eq!(decimal, s.decimal, "decimal representation");
    //         Ok(())
    //     }

    //     #[test]
    //     fn should_calculate_with_missing_day() -> Result<(), TrackerError> {
    //         logger();
    //         let TimeStatus = [
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 7)),
    //                 entries: get_entries(7, 8, 45),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 8)),
    //                 entries: get_entries(8, 8, 45),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 9)),
    //                 entries: get_entries(9, 8, 45),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 10)),
    //                 entries: get_entries(10, 10, 45),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 11)),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 12)),
    //                 ..Default::default()
    //             },
    //             TimeStatus {
    //                 date: Some(Local.ymd(2022, 3, 13)),
    //                 ..Default::default()
    //             },
    //         ]
    //         .to_vec();
    //         let settings = get_settings();
    //         let TimeStatus_weekly = TimeStatusWeekly {
    //             entries: TimeStatus,
    //             week: 10,
    //         };

    //         let mut b = TimeStatusWeekly::builder();
    //         let s = b.data(TimeStatus_weekly).settings(settings).build()?;

    //         log::debug!("{}", s);

    //         let week = 10;
    //         let total = TimeStatus::from(Duration::hours(34));
    //         let overtime = TimeStatus::from(Duration::hours(-6));
    //         let decimal = 34.0;

    //         assert_eq!(week, s.week);
    //         assert_eq!(total, s.total);
    //         assert_eq!(overtime, s.overtime);
    //         assert_eq!(decimal, s.decimal);
    //         Ok(())
    //     }
    // }
}
