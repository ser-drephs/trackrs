use std::fmt::Error;

use chrono::{ Datelike, Duration, Local, Timelike };
use colored::{ ColoredString, Colorize };

use crate::{ Action, Configuration, Entry, StatusError, Timesheet, BreakThresholdExtensions };

struct Daily {
    timesheet: Timesheet,
    configuration: Configuration,
}

impl Daily {
    pub fn new(timesheet: Timesheet, configuration: Configuration) -> Self {
        Daily { timesheet, configuration }
    }

    fn fmt_duration(duration: Duration, reverse: bool) -> ColoredString {
        let zero_dur = Duration::minutes(0);

        if reverse {
            match duration.partial_cmp(&zero_dur.into()).unwrap() {
                std::cmp::Ordering::Greater => format!("-{}", duration).bright_green(),
                std::cmp::Ordering::Equal => format!("{}", duration).normal(),
                std::cmp::Ordering::Less =>
                    format!("+{}", duration.checked_mul(-1).unwrap()).bright_red(),
            }
        } else {
            match duration.partial_cmp(&zero_dur.into()).unwrap() {
                std::cmp::Ordering::Greater => format!("+{}", duration).bright_green(),
                std::cmp::Ordering::Equal => format!("+{}", duration).normal(),
                std::cmp::Ordering::Less =>
                    format!("-{}", duration.checked_mul(-1).unwrap()).bright_red(),
            }
        }
    }
}

impl std::fmt::Display for Daily {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let spacing = 13;
        let start = self.timesheet.get_start();
        let end = self.timesheet.get_end();

        let weekday = Local::now().weekday();
        let expected_worktime = self.configuration.workperday.into_duration(weekday);
        let expected_break = self.configuration.limits.limit_by_duration(&expected_worktime).unwrap_or(Duration::zero());

        let online = self.timesheet.calculate_online();
        let break_duration = self.timesheet.calculate_break();
        let remaining_break = expected_break - break_duration;
        let overtime = self.timesheet.calculate_overtime(expected_worktime);

        // TODO: continue with formating the message
        let mut message = String::new();
        message.push_str(
            &format!(
                "{:width$}{} ({})",
                "Work time:",
                start,
                Daily::fmt_duration(overtime, false),
                width = spacing
            )
        );
        message.push_str(
            &format!("{:width$}{}", "Online time:", Daily::fmt_duration(online, false), width = spacing)
        );
        message.push_str(
            &format!("{:width$}{} ({})", "Break:", break_duration, Daily::fmt_duration(remaining_break, true), width = spacing)
        );
        message.push_str(
            &format!("{:width$}{}", "Started:", start, width = spacing)
        );
        if self.timesheet.has_end() {
            message.push_str(
                &format!("{:width$}{}", "End:", end, width = spacing)
            );
        } else {
            // This hack is required because in the relative time is know in the current context.
            // A time format like 25:15 doesn't make sense here, whereas 01:15 is understandable in this context.
            let estimated_end = self.timesheet.calculate_estimated_end(expected_worktime, expected_break);
            let hours = estimated_end.hour() % 24;
            let end_fmt = format!("{:0>2}:{:0>2} (est.)", hours, estimated_end.minute()).bright_yellow();
            message.push_str(
                &format!("{:width$}{}", "End (est.):", end_fmt, width = spacing)
            );
        }
        write!(f, "{}", message)
    }
}
