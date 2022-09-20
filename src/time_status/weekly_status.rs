use std::{fmt::Display, ops::Mul};

use chrono::Duration;
use colored::Colorize;

use crate::{time_status::WeeklyBuilder, TimeStatus};

#[derive(Clone, Default, Debug)]
pub struct TimeStatusWeekly {
    pub week: i8,
    pub total: TimeStatus,
    pub overtime: TimeStatus,
    pub decimal: f64,
}

impl TimeStatusWeekly {
    pub fn builder<'a>() -> WeeklyBuilder<'a> {
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
    use crate::test_utils::init;
    use colored::control::ShouldColorize;
    use std::ops::Add;

    #[test]
    fn status_overtime() {
        init();
        let s = TimeStatusWeekly {
            week: 23,
            total: TimeStatus::from(Duration::hours(41).add(Duration::minutes(22))),
            overtime: TimeStatus::from(Duration::minutes(82)),
            decimal: 42.5,
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
        init();
        let s = TimeStatusWeekly {
            week: 23,
            total: TimeStatus::from(Duration::hours(40)),
            overtime: TimeStatus::from(Duration::minutes(0)),
            decimal: 40.0,
        };
        log::debug!("{}", s);

        assert_eq!(
            " Week       | Work time  | Overtime   | Decimal   \n ---------- | ---------- | ---------- | ----------\n         23 |      40:00 |      00:00 |      40.00\n",
            format!("{}", s)
        );
    }

    #[test]
    fn status_less() {
        init();
        let s = TimeStatusWeekly {
            week: 23,
            total: TimeStatus::from(Duration::hours(38).add(Duration::minutes(22))),
            overtime: TimeStatus::from(Duration::minutes(-98)),
            decimal: 38.3,
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
