use std::{ cmp::Ordering, fmt::{ Display, Error, Formatter } };

use colored::Colorize;
use time::{ Duration, OffsetDateTime };
use super::{ output::Output, StatusError };

pub struct Daily<'a> {
    pub start_time: &'a OffsetDateTime,
    pub end_time: &'a OffsetDateTime,
    pub end_expected: &'a OffsetDateTime,
    pub break_time: Option<&'a OffsetDateTime>,
    pub remaining_duration: &'a Duration,
    pub break_duration: &'a Duration,
    pub break_expected: &'a Duration,
    pub work_duration: &'a Duration,
}

impl Daily<'_> {
    fn format_work_time(&self) -> Result<String, StatusError> {
        let remaining = self.remaining_duration.checked_mul(-1).unwrap();
        let (repr, ordering) = Output::format_duration_to_zero(&remaining)?;

        let diff = match ordering {
            Ordering::Greater => repr.bright_green(),
            Ordering::Equal => repr.normal(),
            Ordering::Less => repr.bright_red(),
        };
        Ok(
            Output::format_line(
                "Work time:",
                Output::format_duration(self.work_duration).normal(),
                Some(diff)
            )
        )
    }

    fn format_online_time(&self) -> Result<String, StatusError> {
        let online = *self.end_time - *self.start_time;
        let repr = Output::format_duration(&online);
        Ok(Output::format_line("Online time:", repr.normal(), None))
    }

    fn format_break(&self) -> Result<String, StatusError> {
        let break_diff = self.break_duration.checked_sub(*self.break_expected).unwrap();
        let (repr, ordering) = Output::format_duration_to_zero(&break_diff)?;

        let diff = match ordering {
            Ordering::Greater => repr.bright_yellow(),
            Ordering::Equal => repr.normal(),
            Ordering::Less => repr.bright_red(),
        };
        Ok(
            Output::format_line(
                "Break:",
                Output::format_duration(&self.break_duration).normal(),
                Some(diff)
            )
        )
    }

    fn format_break_time(&self) -> Result<String, StatusError> {
        let mut format_break_time = "".to_string();
        let mut format_break_end = "".to_string();

        if self.break_time.is_some() {
            let break_time = &self.break_time.unwrap();
            let break_end = break_time.checked_add(*self.break_duration).unwrap();
            format_break_time = Output::format_time(&break_time);
            format_break_end = Output::format_time(&break_end);
        }

        Ok(
            Output::format_line(
                "Break taken:",
                format!("{} - {}", format_break_time, format_break_end).normal(),
                None
            )
        )
    }

    fn format_start_time(&self) -> Result<String, StatusError> {
        let repr = Output::format_time(self.start_time);
        Ok(Output::format_line("Started:", repr.normal(), None))
    }

    fn format_end_time(&self) -> Result<String, StatusError> {
        let ended =
            self.remaining_duration.partial_cmp(&Duration::minutes(0)).unwrap() ==
            Ordering::Greater;
        let repr = if ended {
            Output::format_time(self.end_time).bright_green()
        } else {
            format!("{} (est.)", Output::format_time(self.end_expected)).bright_yellow()
        };
        Ok(Output::format_line("End:", repr, None))
    }
}

impl Display for Daily<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.format_work_time().unwrap(),
            self.format_online_time().unwrap(),
            self.format_break().unwrap(),
            self.format_break_time().unwrap(),
            self.format_start_time().unwrap(),
            self.format_end_time().unwrap()
        )
    }
}

#[cfg(test)]
mod tests {
    use colored::control::ShouldColorize;
    use time::{ macros::datetime, Duration, OffsetDateTime };

    use crate::test::{ self };

    use super::Daily;

    #[test]
    fn should_format_work_time_neutral() {
        let daily = Daily {
            start_time: &OffsetDateTime::now_utc(),
            end_time: &OffsetDateTime::now_utc(),
            end_expected: &OffsetDateTime::now_utc(),
            break_time: None,
            remaining_duration: &Duration::minutes(0),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(132),
        };
        let line = daily.format_work_time().unwrap();
        assert_eq!("Work time:   02:12 (±00:00)", format!("{}", line));
    }

    #[test]
    fn should_format_work_time_to_much() {
        let daily = Daily {
            start_time: &OffsetDateTime::now_utc(),
            end_time: &OffsetDateTime::now_utc(),
            end_expected: &OffsetDateTime::now_utc(),
            break_time: None,
            remaining_duration: &Duration::minutes(20),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(40),
            work_duration: &Duration::minutes(132),
        };
        let line = daily.format_work_time().unwrap();
        let exp_diff = "-00:20";
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!(
                    "Work time:   02:12 ({}{}{})",
                    test::TERMINAL_GREEN,
                    exp_diff,
                    test::TERMINAL_NEUTRAL
                ),
                line
            );
        } else {
            assert_eq!(format!("Work time:   02:12 ({})", exp_diff), line);
        }
    }

    #[test]
    fn should_format_work_time_to_less() {
        let daily = Daily {
            start_time: &OffsetDateTime::now_utc(),
            end_time: &OffsetDateTime::now_utc(),
            end_expected: &OffsetDateTime::now_utc(),
            break_time: None,
            remaining_duration: &Duration::minutes(-20),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(50),
            work_duration: &Duration::minutes(132),
        };
        let line = daily.format_work_time().unwrap();
        let exp_diff = "+00:20";
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!(
                    "Work time:   02:12 ({}{}{})",
                    test::TERMINAL_RED,
                    exp_diff,
                    test::TERMINAL_NEUTRAL
                ),
                line
            );
        } else {
            assert_eq!(format!("Work time:   02:12 ({})", exp_diff), line);
        }
    }

    #[test]
    fn should_format_online_time() {
        let daily = Daily {
            start_time: &datetime!(2025-02-03 08:00 UTC),
            end_time: &datetime!(2025-02-03 15:00 UTC),
            end_expected: &datetime!(2025-02-03 16:00 UTC),
            break_time: None,
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_online_time().unwrap();
        assert_eq!("Online time: 07:00", line);
    }

    #[test]
    fn should_format_break_neutral() {
        let daily = Daily {
            start_time: &OffsetDateTime::now_utc(),
            end_time: &OffsetDateTime::now_utc(),
            end_expected: &OffsetDateTime::now_utc(),
            break_time: None,
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_break().unwrap();
        assert_eq!("Break:       00:45 (±00:00)", line);
    }

    #[test]
    fn should_format_break_to_much() {
        let daily = Daily {
            start_time: &OffsetDateTime::now_utc(),
            end_time: &OffsetDateTime::now_utc(),
            end_expected: &OffsetDateTime::now_utc(),
            break_time: None,
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(40),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_break().unwrap();
        let exp_diff = "+00:05";
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!(
                    "Break:       00:45 ({}{}{})",
                    test::TERMINAL_YELLOW,
                    exp_diff,
                    test::TERMINAL_NEUTRAL
                ),
                line
            );
        } else {
            assert_eq!(format!("Break:       00:45 ({})", exp_diff), line);
        }
    }

    #[test]
    fn should_format_break_to_less() {
        let daily = Daily {
            start_time: &OffsetDateTime::now_utc(),
            end_time: &OffsetDateTime::now_utc(),
            end_expected: &OffsetDateTime::now_utc(),
            break_time: None,
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(50),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_break().unwrap();
        let exp_diff = "-00:05";
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!(
                    "Break:       00:45 ({}{}{})",
                    test::TERMINAL_RED,
                    exp_diff,
                    test::TERMINAL_NEUTRAL
                ),
                line
            );
        } else {
            assert_eq!(format!("Break:       00:45 ({})", exp_diff), line);
        }
    }

    #[test]
    fn should_format_break_time() {
        let daily = Daily {
            start_time: &datetime!(2025-02-03 08:04 UTC),
            end_time: &datetime!(2025-02-03 15:00 UTC),
            end_expected: &datetime!(2025-02-03 16:00 UTC),
            break_time: Some(&datetime!(2025-02-03 12:00 UTC)),
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(50),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_break_time().unwrap();
        assert_eq!("Break taken: 12:00 - 12:50", line);
    }

    #[test]
    fn should_format_break_time_no_break() {
        let daily = Daily {
            start_time: &datetime!(2025-02-03 08:04 UTC),
            end_time: &datetime!(2025-02-03 15:00 UTC),
            end_expected: &datetime!(2025-02-03 16:00 UTC),
            break_time: None,
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(50),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_break_time().unwrap();
        assert_eq!("Break taken:  - ", line);
    }

    #[test]
    fn should_format_start_time() {
        let daily = Daily {
            start_time: &datetime!(2025-02-03 08:04 UTC),
            end_time: &datetime!(2025-02-03 15:00 UTC),
            end_expected: &datetime!(2025-02-03 16:00 UTC),
            break_time: None,
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_start_time().unwrap();
        assert_eq!("Started:     08:04", line);
    }

    #[test]
    fn should_format_end_time_est() {
        let daily = Daily {
            start_time: &datetime!(2025-02-03 08:04 UTC),
            end_time: &datetime!(2025-02-03 15:00 UTC),
            end_expected: &datetime!(2025-02-03 16:14 UTC),
            break_time: None,
            remaining_duration: &Duration::minutes(-60),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_end_time().unwrap();
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!(
                    "End:         {}16:14 (est.){}",
                    test::TERMINAL_YELLOW,
                    test::TERMINAL_NEUTRAL
                ),
                line
            );
        } else {
            assert_eq!("End:         16:14 (est.)", line);
        }
    }

    #[test]
    fn should_format_end_time() {
        let daily = Daily {
            start_time: &datetime!(2025-02-03 08:04 UTC),
            end_time: &datetime!(2025-02-03 18:12 UTC),
            end_expected: &datetime!(2025-02-03 16:14 UTC),
            break_time: None,
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let line = daily.format_end_time().unwrap();
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!("End:         {}18:12{}", test::TERMINAL_GREEN, test::TERMINAL_NEUTRAL),
                line
            );
        } else {
            assert_eq!("End:         18:12", line);
        }
    }

    #[test]
    fn should_format_status() {
        let daily = Daily {
            start_time: &datetime!(2025-02-03 08:04 UTC),
            end_time: &datetime!(2025-02-03 18:12 UTC),
            end_expected: &datetime!(2025-02-03 16:14 UTC),
            break_time: Some(&datetime!(2025-02-03 12:12 UTC)),
            remaining_duration: &Duration::minutes(12),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(80),
        };
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!(
                    "Work time:   01:20 ({}-00:12{})\nOnline time: 10:08\nBreak:       00:45 (±00:00)\nBreak taken: 12:12 - 12:57\nStarted:     08:04\nEnd:         {}18:12{}",
                    test::TERMINAL_RED,
                    test::TERMINAL_NEUTRAL,
                    test::TERMINAL_GREEN,
                    test::TERMINAL_NEUTRAL
                ),
                format!("{}", daily)
            );
        } else {
            assert_eq!(
                "Work time:   01:20 (-00:12)\nOnline time: 10:08\nBreak:       00:45 (±00:00)\nBreak taken: 12:12 - 12:57\nStarted:     08:04\nEnd:         18:12",
                format!("{}", daily)
            );
        }
    }
}
