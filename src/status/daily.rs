use std::fmt::{ Display, Error, Formatter };

use time::{ Duration, OffsetDateTime };
use crate::config::BreakLimitExtensions;
use crate::output;
use crate::{ config::Configuration, models::Timesheet };

pub struct Daily<'a> {
    // Duration of work time, breaks are not part of work time
    pub work_duration: &'a Duration,
    // Remaining work time, takes breaks into account
    pub remaining_work_duration: &'a Duration,
    // Online time - which is total time of tracking regardless of breaks
    pub online_duration: &'a Duration,
    // Time of "first" break with "end" time of break.
    pub break_time: Option<&'a OffsetDateTime>,
    // Cumulation of all breaks which is added to break time end value. Used to ease up reporting.
    pub break_duration: Option<&'a Duration>,
    // remaining break time for the expected work time today.
    pub remaining_break_duration: Option<&'a Duration>,
    // start time of tracking
    pub start_time: &'a OffsetDateTime,
    // end of day time - takes expected work time into account.
    pub end_time: &'a OffsetDateTime,

    // pub end_expected: &'a OffsetDateTime,
    // pub break_expected: &'a Duration,
}

impl Daily<'_> {
    pub fn build(c: &Configuration, t: &mut Timesheet) -> Self {
        t.sort();
        let expected_work_time = c.workperday.into_duration(OffsetDateTime::now_utc().weekday());
        let expected_break = c.limits.limit_by_start(&c, &expected_work_time);
        let start_time = &t.start_time();
        let now = OffsetDateTime::now_utc();
        let remaining_break = Daily::calculate_remaining_break(
            t.break_duration().as_ref(),
            expected_break
        ).as_ref();
        Daily {
            work_duration: &t.work_time(),
            remaining_work_duration: &t.remaining_work_time(expected_work_time),
            online_duration: &(now - *start_time),
            break_time: t.break_time().as_ref(),
            break_duration: t.break_duration().as_ref(),
            remaining_break_duration: remaining_break.clone(),
            start_time: &t.start_time(),
            end_time: &t.end_time(),
        }

        // let break_limit = config.limits.limit_by_start();
        // let daily = Daily{
        //     start_time: &t.start_time(),
        //     end_time: &t.end_time(),
        //     break_time: t.break_time().as_ref(),
        //     remaining_duration: &t.remaining_time(expected),
        //     break_duration: &t.break_duration(),
        //     work_duration: &t.work_time(),
        //     end_expected: &t.end_time().checked_add(expected).unwrap(),
        //     break_expected: todo!(),
        // };
    }

    fn calculate_remaining_break(
        break_duration: Option<&Duration>,
        expected_break_duration: Option<Duration>
    ) -> Option<Duration> {
        if break_duration.is_some() && expected_break_duration.is_some() {
            break_duration.unwrap().checked_sub(*expected_break_duration.unwrap())
        } else {
            None
        }
    }
}

impl Display for Daily<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let remaining_work = self.remaining_work_duration.gt(&Duration::minutes(0));
        write!(
            f,
            "{}\n{}\n{}\n{}\n{}\n{}",
            output::Daily
                ::format_work_duration(self.work_duration, self.remaining_work_duration)
                .unwrap(),
            output::Daily::format_online_duration(self.online_duration).unwrap(),
            output::Daily
                ::format_break_duration(self.break_duration, self.remaining_break_duration)
                .unwrap(),
            output::Daily::format_break_time(self.break_time, Some(self.break_duration)).unwrap(),
            output::Daily::format_start_time(self.start_time).unwrap(),
            output::Daily::format_end_time(self.end_time, remaining_work).unwrap()
        )
    }
}

#[cfg(test)]
mod tests {
    use colored::control::ShouldColorize;
    use time::macros::datetime;

    use crate::{ config::Configuration, models::{ Action, Entry, Timesheet }, test };

    use super::Daily;

    #[test]
    fn should_format_daily() {
        let config = Configuration::builder().build().unwrap();
        let mut timesheet = Timesheet::new();
        timesheet.add(Entry::new(Action::Start, datetime!(2025-01-01 10:00 UTC)));
        timesheet.add(Entry::new(Action::Break, datetime!(2025-01-01 12:00 UTC)));
        timesheet.add(Entry::new(Action::Start, datetime!(2025-01-01 12:40 UTC)));
        timesheet.add(Entry::new(Action::End, datetime!(2025-01-01 16:00 UTC)));

        let daily = Daily::build(&config, &mut timesheet);
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
