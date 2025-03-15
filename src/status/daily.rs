use std::{ cmp::Ordering, fmt::{ Display, Error, Formatter } };

use colored::Colorize;
use time::{ Duration, OffsetDateTime };
use super::duration_ext::DurationExtensions as dext;

pub struct Daily<'a> {
    pub start_time: &'a OffsetDateTime,
    pub end_time: &'a OffsetDateTime,
    pub end_expected: &'a OffsetDateTime,
    pub remaining_duration: &'a Duration,
    pub break_duration: &'a Duration,
    pub break_expected: &'a Duration,
    pub work_duration: &'a Duration,
}

impl Daily<'_> {
    fn format_remaining_duration(&self, f: &mut Formatter) -> Result<(), Error> {
        let repr = match self.remaining_duration.partial_cmp(&Duration::minutes(0)).unwrap() {
            Ordering::Greater => format!("+{}", dext::format(self.remaining_duration)?).bright_green(),
            Ordering::Equal => format!("+{}", dext::format(self.remaining_duration)?).normal(),
            Ordering::Less =>
                format!("-{}", dext::format(&self.remaining_duration.checked_mul(-1).unwrap())?).bright_red(),
        };
        write!(f, "{}", repr)
    }

    fn format_break_duration(&self/* , f: &mut Formatter*/) -> Result<String,Error> {
        let diff = self.break_expected.checked_sub(*self.break_duration).unwrap();
        let repr = match diff.partial_cmp(&Duration::minutes(0)).unwrap() {
            Ordering::Less => format!("+{}", diff.checked_mul(-1).unwrap()).bright_yellow(),
            Ordering::Equal => format!("+{}", diff).normal(),
            Ordering::Greater => format!("-{}", diff).bright_red(),
        };
        Ok(format!("{:width$}{} ({})", "Break:", self.break_duration, repr, width = 13))
        // write!(f, "{:width$}{} ({})", "Break:", self.break_duration, repr, width = 13)
    }
}

impl Display for Daily<'_> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let zweo = Duration::minutes(0);
        let rd_fmt = self.format_remaining_duration(f)?;

        // let start = self.start.to_owned().unwrap();
        // let end = self.end.to_owned();
        // let temp_end = self.temp_end.to_owned();
        // let r#break = self.r#break.to_owned().unwrap();

        // let worktime = self.worktime.to_owned();
        // let remaining = self.overtime.to_owned();

        // let rm_fmt = match remaining.partial_cmp(&zero_dr.into()).unwrap() {
        // };

        // let bk_fmt = match break_diff.partial_cmp(&zero_dr.into()).unwrap() {
        // };

        // let mut fmt_break_report = "".to_owned();

        // let end_fmt = if end.is_some() {
        //     let f_break = self.f_break.map(StatusTime::from);

        //     if f_break.is_some() {
        //         let e_break = StatusTime::from(
        //             self.f_break
        //                 .unwrap()
        //                 .add(self.r#break.to_owned().unwrap().duration),
        //         );
        //         fmt_break_report = format!(
        //             "\n{:width$}{} - {}",
        //             "Break taken:",
        //             f_break.unwrap(),
        //             e_break,
        //             width = 13
        //         )
        //     };

        //     format!("{}", end.unwrap()).bright_green()
        // } else if temp_end.is_some() && temp_end.as_ref().unwrap() >= &self.est_end {
        //     format!("{}", temp_end.unwrap()).bright_green()
        // } else {
        //     let hours = self.est_end.hours % 24;
        //     // This hack is required because in the relative time is know in the current context.
        //     // A time format like 25:15 doesn't make sense here, whereas 01:15 is understandable in this context.
        //     format!("{:0>2}:{:0>2} (est.)", hours, self.est_end.minutes).bright_yellow()
        // };

        // let line1 = format!(
        //     "{:width$}{} ({})",
        //     "Work time:",
        //     worktime,
        //     rm_fmt,
        //     width = 13
        // );
        // let line2 = format!(
        //     "{:width$}{}",
        //     "Online time:",
        //     self.online.as_ref().unwrap(),
        //     width = 13
        // );
        // let line3 = format!("{:width$}{} ({})", "Break:", r#break, bk_fmt, width = 13);
        // let line4 = fmt_break_report;
        // let line5 = format!("{:width$}{}", "Started:", start, width = 13);
        // let line6 = format!("{:width$}{}", "End:", end_fmt, width = 13);
        // write!(
        //     f,
        //     "{}\n{}\n{}\n{}\n{}\n{}",
        //     line1, line2, line3, line4, line5, line6
        // )
        write!(f, "{}", self.break_duration)
    }
}

#[cfg(test)]
mod tests {
    use colored::control::ShouldColorize;
    use time::{ macros::datetime, Duration, OffsetDateTime };

    use super::Daily;

    #[test]
    fn should_format_break_neutral() {
        let daily = Daily {
            start_time: &OffsetDateTime::now_utc(),
            end_time: &OffsetDateTime::now_utc(),
            end_expected: &OffsetDateTime::now_utc(),
            remaining_duration: &Duration::minutes(1),
            break_duration: &Duration::minutes(45),
            break_expected: &Duration::minutes(45),
            work_duration: &Duration::minutes(1),
        };
        let format_break = daily.format_break_duration().unwrap();
        assert_eq!(
            "Break:       00:45",
            // indoc!(
            //     "Work time:   06:12 (\u{1b}[91m-01:48\u{1b}[0m)
            // Online time: 06:42
            // Break:       00:20 (\u{1b}[91m-00:10\u{1b}[0m)

            // Break taken: 12:03 - 12:23
            // Started:     08:03
            // End:         \u{1b}[92m14:45\u{1b}[0m"
            // ),
            format!("{}", format_break)
        );
        // if ShouldColorize::from_env().should_colorize() {

        // } else {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   06:12 (-01:48)
        //         Online time: 06:42
        //         Break:       00:20 (-00:10)

        //         Break taken: 12:03 - 12:23
        //         Started:     08:03
        //         End:         14:45"
        //         ),
        //         format!("{}", status)
        //     );
        // }
    }
}
