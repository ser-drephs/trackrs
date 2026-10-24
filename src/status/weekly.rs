use std::{ fmt::Display, ops::Mul };

use chrono::{ DateTime, Duration, NaiveDate, Utc };
use colored::Colorize;
use prettytable::{ format, Table };

use crate::{ DailyData, Settings, StatusDisplay, StatusTime, TimeDataWeekly, TrackerError };

/// The status of a week, or of its days within a month, calculated from its timesheets.
#[derive(Clone, Default, Debug)]
pub struct Weekly {
    pub(crate) week: i16,
    pub(crate) total: StatusTime,
    pub(crate) overtime: StatusTime,
    decimal: f64,
    entries: Vec<(NaiveDate, DailyData)>,
    /// Overtime accumulated before the week.
    carry: Option<StatusTime>,
}

impl Weekly {
    /// Status of `data` at `now`, which ends a day that is still running.
    pub fn new(
        data: &TimeDataWeekly,
        settings: &Settings,
        now: DateTime<Utc>
    ) -> Result<Self, TrackerError> {
        let entries = data.day_data(settings, now)?;

        let mut total = StatusTime::default();
        let mut overtime = StatusTime::default();
        for (_, day) in &entries {
            total += day.worktime().to_owned();
            overtime += day.overtime().to_owned();
        }

        log::info!("totally {} working time and {} overtime", total, overtime);

        let decimal = (total.hours() as f64) + (total.minutes() as f64) * (1.0 / 60.0);
        Ok(Weekly { week: data.week, total, overtime, decimal, entries, carry: None })
    }

    /// Show the overtime accumulated before the week in the table.
    pub fn with_carry(mut self, carry: Option<StatusTime>) -> Self {
        self.carry = carry;
        self
    }

    /// First day of the week, or of its days that were loaded.
    pub fn start(&self) -> Option<NaiveDate> {
        self.entries.first().map(|(d, _)| *d)
    }
}

impl StatusDisplay for Weekly {
    fn table(&self) -> String {
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

        if let Some(carry) = &self.carry {
            table.add_row(row!["Carried over", "", "", "", "", "", carry]);
        }
        let cumulated = StatusTime::cumulated(
            self.carry.to_owned().unwrap_or_default(),
            self.entries.iter().map(|(_, s)| s.overtime())
        );
        self.entries.iter().zip(cumulated).for_each(|((date, status), cumulated)| {
            table.add_row(
                row![
                    date.to_owned().format("%a %d %b"),
                    format!("{}", status_unwrap(status.start().cloned())),
                    format!("{}", status_unwrap(status.end().cloned())),
                    format!("{}", status_unwrap(status.r#break().cloned())),
                    format!("{}", status.worktime()),
                    format!("{}", status.overtime()),
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

        table.to_string()
    }
}

impl Display for Weekly {
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

    use crate::{ BreakLimit, Entry, Status, Timesheet };

    fn logger() {
        // std::env::set_var("RUST_LOG", "trace");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// Clock after the days of the tests, which are all ended, so it doesn't matter.
    fn now() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2022, 3, 14, 12, 0, 0).unwrap()
    }

    mod format {
        use colored::control::ShouldColorize;

        use super::*;

        #[test]
        fn status_overtime() {
            logger();
            let s = Weekly {
                week: 23,
                total: StatusTime::from(Duration::hours(41).add(Duration::minutes(22))),
                overtime: StatusTime::from(Duration::minutes(82)),
                decimal: 42.5,
                entries: Vec::new(),
                carry: None,
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
            let s = Weekly {
                week: 23,
                total: StatusTime::from(Duration::hours(40)),
                overtime: StatusTime::from(Duration::minutes(0)),
                decimal: 40.0,
                entries: Vec::new(),
                carry: None,
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
            let s = Weekly {
                week: 23,
                total: StatusTime::from(Duration::hours(38).add(Duration::minutes(22))),
                overtime: StatusTime::from(Duration::minutes(-98)),
                decimal: 38.3,
                entries: Vec::new(),
                carry: None,
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

    mod new {
        use chrono::Utc;

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
                        status: Status::Connect,
                        time: Utc.with_ymd_and_hms(2022, 3, day.into(), 0, 0, 0).unwrap(),
                        note: None,
                    },
                    Entry {
                        status: Status::Break,
                        time: Utc.with_ymd_and_hms(2022, 3, day.into(), 4, 0, 0).unwrap(),
                        note: None,
                    },
                    Entry {
                        status: Status::Connect,
                        time: Utc.with_ymd_and_hms(2022, 3, day.into(), 4, 30, 0).unwrap(),
                        note: None,
                    },
                    Entry {
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
                ].to_vec()
        }

        /// The days of week 10 of 2022, from Monday on, with their timesheets.
        fn weekly(days: Vec<(DateTime<Utc>, Timesheet)>) -> TimeDataWeekly {
            TimeDataWeekly {
                week: 10,
                days: days.into_iter().map(|(date, day)| (date.date_naive(), day)).collect(),
            }
        }

        fn get_time_data(one_day_end: u8, one_day_minute_off: u8) -> TimeDataWeekly {
            weekly(
                vec![
                    (
                        Utc.with_ymd_and_hms(2022, 3, 7, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(7, 8, 30), ..Default::default() }
                    ),
                    (
                        Utc.with_ymd_and_hms(2022, 3, 8, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(8, 8, 30), ..Default::default() }
                    ),
                    (
                        Utc.with_ymd_and_hms(2022, 3, 9, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(9, one_day_end, 30), ..Default::default() }
                    ),
                    (
                        Utc.with_ymd_and_hms(2022, 3, 10, 0, 0, 0).unwrap(),
                        Timesheet {
                            data: get_entries(10, 8, one_day_minute_off),
                            ..Default::default()
                        }
                    ),
                    (
                        Utc.with_ymd_and_hms(2022, 3, 11, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(11, 8, 30), ..Default::default() }
                    ),
                    (Utc.with_ymd_and_hms(2022, 3, 12, 0, 0, 0).unwrap(), Timesheet::default()),
                    (Utc.with_ymd_and_hms(2022, 3, 13, 0, 0, 0).unwrap(), Timesheet::default())
                ]
            )
        }

        #[test]
        fn should_calculate_overtime() -> Result<(), TrackerError> {
            logger();
            let s = Weekly::new(&get_time_data(10, 42), &get_settings(), now())?;

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
            let s = Weekly::new(&get_time_data(9, 45), &get_settings(), now())?;

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
            let s = Weekly::new(&get_time_data(6, 17), &get_settings(), now())?;

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
            let time_data = weekly(
                vec![
                    (
                        Utc.with_ymd_and_hms(2022, 3, 7, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(7, 8, 45), ..Default::default() }
                    ),
                    (
                        Utc.with_ymd_and_hms(2022, 3, 8, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(8, 8, 45), ..Default::default() }
                    ),
                    (
                        Utc.with_ymd_and_hms(2022, 3, 9, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(9, 8, 45), ..Default::default() }
                    ),
                    (
                        Utc.with_ymd_and_hms(2022, 3, 10, 0, 0, 0).unwrap(),
                        Timesheet { data: get_entries(10, 10, 45), ..Default::default() }
                    ),
                    (Utc.with_ymd_and_hms(2022, 3, 11, 0, 0, 0).unwrap(), Timesheet::default()),
                    (Utc.with_ymd_and_hms(2022, 3, 12, 0, 0, 0).unwrap(), Timesheet::default()),
                    (Utc.with_ymd_and_hms(2022, 3, 13, 0, 0, 0).unwrap(), Timesheet::default())
                ]
            );
            let s = Weekly::new(&time_data, &get_settings(), now())?;

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
                .map(|(_, d)| d.overtime().to_string())
                .collect();
            assert_eq!(vec!["00:00", "00:00", "00:00", "02:00", "-08:00", "00:00", "00:00"], rows);
            Ok(())
        }

        #[test]
        fn should_reject_day_without_connect() {
            let time_data = weekly(
                vec![
                    (
                        Utc.with_ymd_and_hms(2022, 3, 7, 0, 0, 0).unwrap(),
                        Timesheet {
                            data: vec![Entry {
                                status: Status::End,
                                time: Utc.with_ymd_and_hms(2022, 3, 7, 8, 0, 0).unwrap(),
                                note: None,
                            }],
                            ..Default::default()
                        }
                    )
                ]
            );

            assert!(Weekly::new(&time_data, &get_settings(), now()).is_err());
        }

        #[test]
        fn should_start_with_the_first_day() -> Result<(), TrackerError> {
            let s = Weekly::new(&get_time_data(9, 45), &get_settings(), now())?;

            assert_eq!(NaiveDate::from_ymd_opt(2022, 3, 7), s.start());
            Ok(())
        }

        #[test]
        fn should_have_no_start_without_days() -> Result<(), TrackerError> {
            let s = Weekly::new(&TimeDataWeekly::default(), &get_settings(), now())?;

            assert_eq!(None, s.start());
            Ok(())
        }

        #[test]
        fn should_show_a_row_for_each_day_and_the_total_in_the_table() -> Result<(), TrackerError> {
            logger();
            let s = Weekly::new(&get_time_data(10, 42), &get_settings(), now())?;
            let table = s.table();

            for day in ["Mon 07 Mar", "Tue 08 Mar", "Wed 09 Mar", "Thu 10 Mar", "Fri 11 Mar", "Sat 12 Mar", "Sun 13 Mar"] {
                assert!(table.contains(day), "{day} in {table}");
            }
            assert!(table.contains("Total week 10"), "{table}");
            assert!(table.contains("40:57"), "{table}");
            assert!(!table.contains("Carried over"), "{table}");
            Ok(())
        }

        #[test]
        fn should_add_the_carry_to_the_table_and_the_cumulated_overtime() -> Result<(), TrackerError> {
            let carry = StatusTime::from(Duration::hours(2));
            let s = Weekly::new(&get_time_data(9, 45), &get_settings(), now())?.with_carry(
                Some(carry)
            );
            let table = s.table();

            assert!(table.contains("Carried over"), "{table}");
            let first_row = table.lines().find(|l| l.contains("Carried over")).unwrap();
            assert!(first_row.contains("02:00"), "{first_row}");
            // Monday is 15 minutes short, and the week as a whole is on point, so the cumulated
            // overtime starts below the carry and ends at it.
            let monday = table.lines().find(|l| l.contains("Mon 07 Mar")).unwrap();
            assert!(monday.trim_end().ends_with("01:45"), "{monday}");
            let sunday = table.lines().find(|l| l.contains("Sun 13 Mar")).unwrap();
            assert!(sunday.trim_end().ends_with("02:00"), "{sunday}");
            Ok(())
        }

        #[test]
        fn should_remove_the_carry() -> Result<(), TrackerError> {
            let carry = Some(StatusTime::from(Duration::hours(2)));
            let s = Weekly::new(&get_time_data(9, 45), &get_settings(), now())?
                .with_carry(carry)
                .with_carry(None);

            assert!(!s.table().contains("Carried over"));
            Ok(())
        }
    }
}
