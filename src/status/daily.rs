use std::fmt::Error;

use chrono::Duration;
use colored::{ ColoredString, Colorize };

use crate::{ Action, Entry, StatusError, Timesheet };

struct Daily {
    timesheet: Timesheet,
}

impl Daily {
    pub fn new(timesheet: Timesheet) -> Self {
        Daily { timesheet }
    }

    fn fmt_duration(duration: Duration, reverse: bool) -> ColoredString {
        let zero_dr = Duration::minutes(0);

        if reverse {
            match duration.partial_cmp(&zero_dr.into()).unwrap() {
                std::cmp::Ordering::Greater => format!("-{}", duration).bright_green(),
                std::cmp::Ordering::Equal => format!("{}", duration).normal(),
                std::cmp::Ordering::Less =>
                    format!("+{}", duration.checked_mul(-1).unwrap()).bright_red(),
            }
        } else {
            match duration.partial_cmp(&zero_dr.into()).unwrap() {
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
        let start = self.timesheet
            .data()
            .iter()
            .find(|x| x.is_action(Action::Start));
        if start.is_none() {
            // TODO: check to throw custom error of type statuserror...
            return Err(Error);
        }

        // TODO: continue with formating the message
        let mut message = String::new();

        let fmt_remaining_worktime = Daily::fmt_duration(remaining, false);

        message.push_str(
            &format!("{:width$}{} ({})", "Work time:", start.unwrap(), fmt_remaining_worktime, width = 13)
        );

        let end = self.timesheet
            .data()
            .iter()
            .find(|x| x.is_action(Action::End));

        let first_break = self.timesheet
            .data()
            .iter()
            .find(|x| x.is_action(Action::Break));

        // TODO get expected break time from configuration
        let break_taken = self.timesheet.get_break(Duration::seconds(0));

        let temp_end = self.temp_end.to_owned();
        let r#break = self.r#break.to_owned().unwrap();

        let zero_dr = Duration::minutes(0);
        let worktime = self.worktime.to_owned();
        let remaining = self.overtime.to_owned();

        let fmt_break_taken = Daily::fmt_duration(break_taken, true);

        // let break_diff = self.exp_break.to_owned().unwrap() - self.r#break.to_owned().unwrap();

        let mut fmt_break_report = "".to_owned();

        let end_fmt = if end.is_some() {
            let f_break = self.f_break.map(StatusTime::from);

            if f_break.is_some() {
                let e_break = StatusTime::from(
                    self.f_break.unwrap().add(self.r#break.to_owned().unwrap().duration)
                );
                fmt_break_report = format!(
                    "\n{:width$}{} - {}",
                    "Break taken:",
                    f_break.unwrap(),
                    e_break,
                    width = 13
                );
            }

            format!("{}", end.unwrap()).bright_green()
        } else if temp_end.is_some() && temp_end.as_ref().unwrap() >= &self.est_end {
            format!("{}", temp_end.unwrap()).bright_green()
        } else {
            let hours = self.est_end.hours % 24;
            // This hack is required because in the relative time is know in the current context.
            // A time format like 25:15 doesn't make sense here, whereas 01:15 is understandable in this context.
            format!("{:0>2}:{:0>2} (est.)", hours, self.est_end.minutes).bright_yellow()
        };

        let line1 = format!(
            "{:width$}{} ({})",
            "Work time:",
            worktime,
            fmt_remaining_worktime,
            width = 13
        );
        let line2 = format!(
            "{:width$}{}",
            "Online time:",
            self.online.as_ref().unwrap(),
            width = 13
        );
        let line3 = format!("{:width$}{} ({})", "Break:", r#break, fmt_break_taken, width = 13);
        let line4 = fmt_break_report;
        let line5 = format!("{:width$}{}", "Started:", start, width = 13);
        let line6 = format!("{:width$}{}", "End:", end_fmt, width = 13);
        write!(f, "{}\n{}\n{}\n{}\n{}\n{}", line1, line2, line3, line4, line5, line6)
    }
}
