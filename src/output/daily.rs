use std::cmp::Ordering;

use colored::Colorize;
use time::{ Duration, OffsetDateTime };
use super::{ output::Output, StatusError };

pub struct Daily {}

impl Daily {
    pub fn format_work_duration(
        work_duration: &Duration,
        remaining: &Duration
    ) -> Result<String, StatusError> {
        // representation of remaining should be -x if it is "positive"
        let inv = remaining.checked_mul(-1).unwrap();
        let (repr, ordering) = Output::format_duration_to_zero(&inv)?;

        let diff = match ordering {
            Ordering::Greater => repr.bright_red(),
            Ordering::Equal => repr.normal(),
            Ordering::Less => repr.bright_green(),
        };
        Ok(
            Output::format_line(
                "Work time:",
                Output::format_duration(work_duration).normal(),
                Some(diff)
            )
        )
    }

    pub fn format_online_duration(online_duration: &Duration) -> Result<String, StatusError> {
        let repr = Output::format_duration(online_duration);
        Ok(Output::format_line("Online time:", repr.normal(), None))
    }

    pub fn format_break_duration(
        break_duration: &Duration,
        remaining: &Duration
    ) -> Result<String, StatusError> {
        let (repr, ordering) = Output::format_duration_to_zero(remaining)?;

        let diff = match ordering {
            Ordering::Greater => repr.bright_yellow(),
            Ordering::Equal => repr.normal(),
            Ordering::Less => repr.bright_red(),
        };
        Ok(
            Output::format_line(
                "Break:",
                Output::format_duration(break_duration).normal(),
                Some(diff)
            )
        )
    }

    pub fn format_break_time(
        break_time: Option<&OffsetDateTime>,
        break_duration: Option<&Duration>
    ) -> Result<String, StatusError> {
        if break_time.is_some() && break_duration.is_some() {
            let break_time = break_time.unwrap();
            let break_duration = break_duration.unwrap();
            let format_break_time = Output::format_time(break_time);
            let format_break_end = Output::format_time(
                &break_time.checked_add(*break_duration).unwrap()
            );
            Ok(
                Output::format_line(
                    "Break taken:",
                    format!("{} - {}", format_break_time, format_break_end).normal(),
                    None
                )
            )
        } else {
            Ok("".to_string())
        }
    }

    pub fn format_start_time(start_time: &OffsetDateTime) -> Result<String, StatusError> {
        let repr = Output::format_time(start_time);
        Ok(Output::format_line("Started:", repr.normal(), None))
    }

    pub fn format_end_time(
        end_time: &OffsetDateTime,
        has_remaining: bool
    ) -> Result<String, StatusError> {
        let repr = if has_remaining {
            Output::format_time(end_time).bright_yellow()
        } else {
            Output::format_time(end_time).bright_green()
        };
        Ok(Output::format_line("End:", repr, None))
    }
}

#[cfg(test)]
mod tests {
    use colored::control::ShouldColorize;
    use time::{ macros::datetime, Duration };

    use crate::test::{ self };

    use super::Daily;

    #[test]
    fn should_format_work_time_neutral() {
        let line = Daily::format_work_duration(
            &Duration::minutes(132),
            &Duration::minutes(0)
        ).unwrap();
        assert_eq!("Work time:   02:12 (±00:00)", format!("{}", line));
    }

    #[test]
    fn should_format_work_time_to_much() {
        let line = Daily::format_work_duration(
            &Duration::minutes(132),
            &Duration::minutes(20)
        ).unwrap();
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
        let line = Daily::format_work_duration(
            &Duration::minutes(132),
            &Duration::minutes(-20)
        ).unwrap();
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
        let line = Daily::format_online_duration(&Duration::hours(7)).unwrap();
        assert_eq!("Online time: 07:00", line);
    }

    #[test]
    fn should_format_break_neutral() {
        let line = Daily::format_break_duration(
            &Duration::minutes(45),
            &Duration::minutes(0)
        ).unwrap();
        assert_eq!("Break:       00:45 (±00:00)", line);
    }

    #[test]
    fn should_format_break_to_much() {
        let line = Daily::format_break_duration(
            &Duration::minutes(45),
            &Duration::minutes(5)
        ).unwrap();
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
        let line = Daily::format_break_duration(
            &Duration::minutes(45),
            &Duration::minutes(-5)
        ).unwrap();
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
        let line = Daily::format_break_time(
            Some(&datetime!(2025-02-03 12:00 UTC)),
            Some(&Duration::minutes(50))
        ).unwrap();
        assert_eq!("Break taken: 12:00 - 12:50", line);
    }

    #[test]
    fn should_format_break_time_no_break() {
        let line = Daily::format_break_time(Some(&datetime!(2025-02-03 12:00 UTC)), None).unwrap();
        assert_eq!("", line);
    }

    #[test]
    fn should_format_start_time() {
        let line = Daily::format_start_time(&datetime!(2025-02-03 08:04 UTC)).unwrap();
        assert_eq!("Started:     08:04", line);
    }

    #[test]
    fn should_format_end_time_est() {
        let line = Daily::format_end_time(&datetime!(2025-02-03 16:14 UTC), true).unwrap();
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!("End:         {}16:14{}", test::TERMINAL_YELLOW, test::TERMINAL_NEUTRAL),
                line
            );
        } else {
            assert_eq!("End:         16:14", line);
        }
    }

    #[test]
    fn should_format_end_time() {
        let line = Daily::format_end_time(&datetime!(2025-02-03 18:12 UTC), false).unwrap();
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                format!("End:         {}18:12{}", test::TERMINAL_GREEN, test::TERMINAL_NEUTRAL),
                line
            );
        } else {
            assert_eq!("End:         18:12", line);
        }
    }
}
