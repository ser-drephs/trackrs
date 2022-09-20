use crate::{TimeStatus, time_status::DailyBuilder};
use chrono::Duration;
use colored::Colorize;
use std::ops::Mul;

#[derive(Default, Clone, Debug)]
pub struct TimeStatusDaily {
    /// start time
    ///
    /// Start: {<start>}
    pub start: TimeStatus,
    /// end time
    ///
    /// End: {<end>}
    pub end: TimeStatus,
    /// has actually end time, if false then a temporary end time was used.
    ///
    /// Will result in displaying estimated end instead of end.
    /// End: {} (est.)
    pub has_end: bool,
    /// break time
    ///
    /// Break: {<break>} ({})
    pub r#break: TimeStatus,
    /// online time
    ///
    /// Online time: {<online>}
    pub online: TimeStatus,
    /// expected work time
    pub exworktime: TimeStatus,
    /// first break
    ///
    /// Break taken: {<fbreak>} - {}
    pub fbreak: TimeStatus,
    /// expected break time
    ///
    /// Break; {} ({<exbreak - break>})
    pub exbreak: TimeStatus,
    /// calculated break time
    ///
    /// Break taken: {} - {<fbreak + cbreak>}
    pub cbreak: TimeStatus,
    /// work time
    ///
    /// Work time: {<worktime>} ({})
    pub worktime: TimeStatus,
    /// over time
    ///
    /// Work time: {} ({<overtime>})
    pub overtime: TimeStatus,
    /// estimated end time
    ///
    /// With has end set, will display this value
    /// End: {<esend>} (est.)
    pub esend: TimeStatus,
}

impl TimeStatusDaily {
    pub fn builder<'a>() -> DailyBuilder<'a> {
        DailyBuilder::default()
    }
}

impl std::fmt::Display for TimeStatusDaily {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let width = 13;
        let zero_dr = Duration::minutes(0);
        let overtime = self.overtime.clone();
        let part_overtime = match overtime.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Greater => format!("+{}", overtime).bright_green(),
            std::cmp::Ordering::Equal => format!("+{}", overtime).normal(),
            std::cmp::Ordering::Less => format!("-{}", overtime.mul(-1)).bright_red(),
        };

        let fmt_worktime = format!(
            "{0:width$}{1} ({2})",
            "Work time:", self.worktime, part_overtime
        );
        let fmt_online = format!("{0:width$}{1}", "Online time:", self.online);

        let break_diff = self.exbreak.clone() - self.r#break.clone();
        let part_break_diff = match break_diff.partial_cmp(&zero_dr.into()).unwrap() {
            std::cmp::Ordering::Less => format!("+{}", break_diff.mul(-1)).bright_yellow(),
            std::cmp::Ordering::Equal => format!("+{}", break_diff).normal(),
            std::cmp::Ordering::Greater => format!("-{}", break_diff).bright_red(),
        };
        let fmt_break = format!(
            "{0:width$}{1} ({2})",
            "Break:", self.r#break, part_break_diff
        );

        let part_break_report = self.fbreak.clone() + self.cbreak.clone();

        let fmt_break_report = if self.has_end {
            format!(
                "\n{0:width$}{1} - {2}",
                "Break taken:", self.fbreak, part_break_report
            )
        } else {
            "".to_string()
        };

        let fmt_start = format!("{0:width$}{1}", "Started:", self.start);

        let part_end = if self.has_end {
            format!("{}", self.end).bright_green()
        } else {
            let hours = self.esend.hours % 24;
            // This hack is required because in the relative time is know in the current context.
            // A time format like 25:15 doesn't make sense here, whereas 01:15 is understandable in this context.
            format!("{:0>2}:{:0>2} (est.)", hours, self.esend.minutes).bright_yellow()
        };

        let fmt_end = format!("{0:width$}{1}", "End:", part_end);

        write!(
            f,
            "{}\n{}\n{}\n{}\n{}\n{}",
            fmt_worktime, fmt_online, fmt_break, fmt_break_report, fmt_start, fmt_end
        )
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::test_utils::init;
    use colored::control::ShouldColorize;
    use indoc::indoc;
    use std::ops::Add;

    #[test]
    fn format_with_est_end() {
        init();
        let daily = TimeStatusDaily {
            start: Duration::hours(8).add(Duration::minutes(3)).into(),
            end: Duration::hours(15).into(),
            r#break: Duration::minutes(15).into(),
            cbreak: Duration::minutes(30).into(),
            exbreak: Duration::minutes(45).into(),
            exworktime: Duration::hours(8).add(Duration::minutes(45)).into(),
            has_end: false,
            esend: Duration::hours(16).add(Duration::minutes(45)).into(),
            fbreak: Duration::hours(12).into(),
            online: Duration::hours(15).into(),
            overtime: Duration::hours(-2).add(Duration::minutes(-15)).into(),
            worktime: Duration::hours(7).into(),
        };

        log::debug!("{}", daily);

        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (\u{1b}[91m-02:15\u{1b}[0m)
                    Online time: 15:00
                    Break:       00:15 (\u{1b}[91m-00:30\u{1b}[0m)

                    Started:     08:03
                    End:         \u{1b}[93m16:45 (est.)\u{1b}[0m"
                ),
                format!("{}", daily)
            );
        } else {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (-02:15)
                Online time: 15:00
                Break:       00:15 (-00:30)

                Started:     08:03
                End:         16:45"
                ),
                format!("{}", daily)
            );
        }
    }

    #[test]
    fn format_with_overtime() {
        init();
        let daily = TimeStatusDaily {
            start: Duration::hours(8).add(Duration::minutes(3)).into(),
            end: Duration::hours(15).into(),
            r#break: Duration::minutes(45).into(),
            cbreak: Duration::minutes(0).into(),
            exbreak: Duration::minutes(45).into(),
            exworktime: Duration::hours(8).add(Duration::minutes(45)).into(),
            has_end: true,
            esend: Duration::hours(16).add(Duration::minutes(45)).into(),
            fbreak: Duration::hours(12).into(),
            online: Duration::hours(15).into(),
            overtime: Duration::hours(2).add(Duration::minutes(15)).into(),
            worktime: Duration::hours(7).into(),
        };

        log::debug!("{}", daily);

        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (\u{1b}[92m+02:15\u{1b}[0m)
                    Online time: 15:00
                    Break:       00:45 (+00:00)

                    Break taken: 12:00 - 12:45
                    Started:     08:03
                    End:         \u{1b}[92m15:00\u{1b}[0m"
                ),
                format!("{}", daily)
            );
        } else {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (+02:15)
                    Online time: 15:00
                    Break:       00:45 (+00:00)

                    Break taken: 12:00 - 12:45
                    Started:     08:03
                    End:         15:00"
                ),
                format!("{}", daily)
            );
        }
    }

    #[test]
    fn format_with_more_break() {
        init();
        let daily = TimeStatusDaily {
            start: Duration::hours(8).add(Duration::minutes(3)).into(),
            end: Duration::hours(15).into(),
            r#break: Duration::minutes(90).into(),
            cbreak: Duration::minutes(45).into(),
            exbreak: Duration::minutes(45).into(),
            exworktime: Duration::hours(8).add(Duration::minutes(45)).into(),
            has_end: true,
            esend: Duration::hours(16).add(Duration::minutes(45)).into(),
            fbreak: Duration::hours(12).into(),
            online: Duration::hours(15).into(),
            overtime: Duration::hours(2).add(Duration::minutes(15)).into(),
            worktime: Duration::hours(7).into(),
        };

        log::debug!("{}", daily);

        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (\u{1b}[92m+02:15\u{1b}[0m)
                    Online time: 15:00
                    Break:       01:30 (\u{1b}[93m+00:45\u{1b}[0m)

                    Break taken: 12:00 - 13:30
                    Started:     08:03
                    End:         \u{1b}[92m15:00\u{1b}[0m"
                ),
                format!("{}", daily)
            );
        } else {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (+02:15)
                    Online time: 15:00
                    Break:       01:30 (+00:45)

                    Break taken: 12:00 - 13:30
                    Started:     08:03
                    End:         15:00"
                ),
                format!("{}", daily)
            );
        }
    }

    #[test]
    fn format_with_more_break_est_end() {
        init();
        let daily = TimeStatusDaily {
            start: Duration::hours(8).add(Duration::minutes(3)).into(),
            end: Duration::hours(15).into(),
            r#break: Duration::minutes(90).into(),
            cbreak: Duration::minutes(45).into(),
            exbreak: Duration::minutes(45).into(),
            exworktime: Duration::hours(8).add(Duration::minutes(45)).into(),
            has_end: false,
            esend: Duration::hours(16).add(Duration::minutes(45)).into(),
            fbreak: Duration::hours(12).into(),
            online: Duration::hours(15).into(),
            overtime: Duration::hours(2).add(Duration::minutes(15)).into(),
            worktime: Duration::hours(7).into(),
        };

        log::debug!("{}", daily);

        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (\u{1b}[92m+02:15\u{1b}[0m)
                    Online time: 15:00
                    Break:       01:30 (\u{1b}[93m+00:45\u{1b}[0m)

                    Started:     08:03
                    End:         \u{1b}[93m16:45 (est.)\u{1b}[0m"
                ),
                format!("{}", daily)
            );
        } else {
            assert_eq!(
                indoc!(
                    "Work time:   07:00 (+02:15)
                    Online time: 15:00
                    Break:       01:30 (+00:45)

                    Started:     08:03
                    End:         16:45 (est.)"
                ),
                format!("{}", daily)
            );
        }
    }
}
