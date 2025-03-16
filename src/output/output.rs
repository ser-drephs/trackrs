use std::cmp::Ordering;

use colored::ColoredString;
use time::{ Duration, OffsetDateTime };

use super::StatusError;

const WIDTH: usize = 13;

pub struct Output {}

impl Output {
    fn format_duration_to_zero_internal(
        duration: &Duration,
        invert: bool
    ) -> Result<(String, Ordering), StatusError> {
        let ordering = duration.partial_cmp(&Duration::minutes(0));
        let char_gt = if invert { "-" } else { "+" };
        let char_lt = if invert { "+" } else { "-" };
        let repr = match &ordering {
            Some(Ordering::Greater) => format!("{}{}", char_gt, Output::format_duration(&duration)),
            Some(Ordering::Equal) => "±00:00".to_string(),
            Some(Ordering::Less) =>
                format!(
                    "{}{}",
                    char_lt,
                    Output::format_duration(&duration.checked_mul(-1).unwrap())
                ),
            None => {
                return Err(StatusError::ZeroComparisonError {
                    duration: "remaining time".to_string(),
                });
            }
        };
        Ok((repr, ordering.unwrap()))
    }

    pub fn format_duration_to_zero(duration: &Duration) -> Result<(String, Ordering), StatusError> {
        Output::format_duration_to_zero_internal(duration, false)
    }

    pub fn format_line(text: &str, repr: ColoredString, diff: Option<ColoredString>) -> String {
        match diff {
            Some(add) => format!("{:width$}{} ({})", text, repr, add, width = WIDTH),
            None => format!("{:width$}{}", text, repr, width = WIDTH),
        }
    }

    pub fn format_duration(duration: &Duration) -> String {
        let h = duration.whole_hours();
        let m = duration.whole_minutes() - 60 * h;
        format!("{:0>2}:{:0>2}", h, m)
    }

    pub fn format_time(time: &OffsetDateTime) -> String {
        let h = time.hour();
        let m = time.minute();
        format!("{:0>2}:{:0>2}", h, m)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use colored::{ control::ShouldColorize, Colorize };
    use time::{macros::datetime, Duration};

    use crate::{ output::Output, test };

    #[test]
    fn should_format_duration_minutes() {
        let duration = Duration::minutes(45);
        let repr = Output::format_duration(&duration);
        assert_eq!(repr, "00:45")
    }

    #[test]
    fn should_format_duration_single_minutes() {
        let duration = Duration::minutes(1);
        let repr = Output::format_duration(&duration);
        assert_eq!(repr, "00:01")
    }

    #[test]
    fn should_format_duration_hour() {
        let duration = Duration::minutes(60);
        let repr = Output::format_duration(&duration);
        assert_eq!(repr, "01:00")
    }

    #[test]
    fn should_format_duration_hour_minutes() {
        let duration = Duration::minutes(75);
        let repr = Output::format_duration(&duration);
        assert_eq!(repr, "01:15")
    }

    #[test]
    fn should_format_line_with_diff() {
        let text = Output::format_line(
            "Break:",
            "00:20".to_string().normal(),
            Some("Over".to_string().normal())
        );
        assert_eq!(text, "Break:       00:20 (Over)")
    }

    #[test]
    fn should_format_line_no_diff() {
        let text = Output::format_line("Break:", "00:20".to_string().normal(), None);
        assert_eq!(text, "Break:       00:20")
    }

    #[test]
    fn should_format_line_with_color_diff() {
        let text = Output::format_line(
            "Break:",
            "00:20".to_string().normal(),
            Some("Over".to_string().bright_red())
        );
        if ShouldColorize::from_env().should_colorize() {
            assert_eq!(
                text,
                format!("Break:       00:20 ({}Over{})", test::TERMINAL_RED, test::TERMINAL_NEUTRAL)
            )
        } else {
            assert_eq!(text, "Break:       00:20 (Over)")
        }
    }

    #[test]
    fn should_format_duration_greater_than_0() {
        let d = Duration::minutes(10);
        let (repr, ordering) = Output::format_duration_to_zero(&d).unwrap();
        assert_eq!(ordering, Ordering::Greater);
        assert_eq!(repr, "+00:10")
    }

    #[test]
    fn should_format_duration_less_than_0() {
        let d = Duration::minutes(-10);
        let (repr, ordering) = Output::format_duration_to_zero(&d).unwrap();
        assert_eq!(ordering, Ordering::Less);
        assert_eq!(repr, "-00:10")
    }

    #[test]
    fn should_format_duration_eq_0() {
        let d = Duration::minutes(0);
        let (repr, ordering) = Output::format_duration_to_zero(&d).unwrap();
        assert_eq!(ordering, Ordering::Equal);
        assert_eq!(repr, "±00:00")
    }

    #[test]
    fn should_format_time(){
        let t = datetime!(2025-01-02 12:04 UTC);
        let repr = Output::format_time(&t);
        assert_eq!(repr, "12:04")
    }

    #[test]
    fn should_format_time_ext_day(){
        let t = datetime!(2025-01-02 22:04 UTC);
        let e = t.saturating_add(Duration::hours(8));
        let repr = Output::format_time(&e);
        assert_eq!(repr, "06:04")
    }
}
