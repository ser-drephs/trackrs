use chrono::{ DateTime, Duration, FixedOffset, NaiveDate };

use crate::{ Settings, StatusTime, Timesheet, TrackerError };

/// What a day adds up to. A day without entries only has its missing overtime.
#[derive(Clone, Default, Debug)]
pub struct DailyData {
    start: Option<StatusTime>,
    end: Option<StatusTime>,
    r#break: Option<StatusTime>,
    worktime: StatusTime,
    overtime: StatusTime,
}

impl DailyData {
    /// The day `date` added up from its `timesheet` at `now`, which ends a day that is still
    /// running.
    pub fn new(
        date: NaiveDate,
        timesheet: &Timesheet,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<Self, TrackerError> {
        if timesheet.data.is_empty() {
            let expected = settings.workperday.minutes_on(date);
            // A day without entries misses its whole target worktime.
            let overtime = StatusTime::from(Duration::minutes(-i64::from(expected)));
            return Ok(DailyData { overtime, ..Default::default() });
        }

        let day = DailyData {
            start: timesheet.start(),
            end: timesheet.end(),
            r#break: Some(timesheet.break_taken()),
            worktime: timesheet.worktime(settings, now)?,
            overtime: timesheet.overtime(settings, now)?,
        };
        log::info!("got {} working time and {} overtime", day.worktime, day.overtime);
        Ok(day)
    }

    /// Time of the first connect, `None` for a day without entries.
    pub fn start(&self) -> Option<&StatusTime> {
        self.start.as_ref()
    }

    /// Time of the end of the day, `None` for a day that is running or without entries.
    pub fn end(&self) -> Option<&StatusTime> {
        self.end.as_ref()
    }

    /// Total of the breaks taken, `None` for a day without entries.
    pub fn r#break(&self) -> Option<&StatusTime> {
        self.r#break.as_ref()
    }

    pub fn worktime(&self) -> &StatusTime {
        &self.worktime
    }

    pub fn overtime(&self) -> &StatusTime {
        &self.overtime
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ TimeZone, Utc };

    use crate::{ BreakLimit, Entry, Status };

    use super::*;

    fn at(day: u32, hour: u32, minute: u32) -> DateTime<FixedOffset> {
        Utc.with_ymd_and_hms(2022, 1, day, hour, minute, 0).unwrap().fixed_offset()
    }

    fn date(day: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(2022, 1, day).unwrap()
    }

    fn settings() -> Settings {
        Settings {
            limits: vec![BreakLimit { start: 6 * 60, minutes: 30 }],
            ..Default::default()
        }
    }

    /// The day `day` of January 2022 with the given (status, hour, minute) entries.
    fn timesheet(day: u32, entries: &[(Status, u32, u32)]) -> Timesheet {
        let data = entries
            .iter()
            .map(|(status, h, m)| Entry {
                status: status.to_owned(),
                time: at(day, *h, *m),
                note: None,
            })
            .collect();
        Timesheet { data, ..Default::default() }
    }

    #[test]
    fn should_add_up_a_day_with_entries() -> Result<(), TrackerError> {
        let timesheet = timesheet(
            17,
            &[
                (Status::Connect, 8, 0),
                (Status::Break, 12, 0),
                (Status::Connect, 12, 30),
                (Status::End, 17, 0),
            ]
        );
        let day = DailyData::new(date(17), &timesheet, &settings(), at(21, 12, 0))?;

        assert!(day.start().is_some());
        assert!(day.end().is_some());
        assert_eq!("00:30", day.r#break().unwrap().to_string());
        assert_eq!("08:30", day.worktime().to_string());
        assert_eq!("00:30", day.overtime().to_string());
        Ok(())
    }

    #[test]
    fn should_only_miss_the_target_worktime_on_a_day_without_entries() -> Result<(), TrackerError> {
        // Tuesday has a target of 8 hours.
        let day = DailyData::new(date(18), &Timesheet::default(), &settings(), at(21, 12, 0))?;

        assert_eq!("-08:00", day.overtime().to_string());
        assert_eq!("00:00", day.worktime().to_string());
        assert!(day.start().is_none() && day.end().is_none() && day.r#break().is_none());
        Ok(())
    }

    #[test]
    fn should_not_miss_anything_on_a_day_without_target() -> Result<(), TrackerError> {
        // Saturday has no target.
        let day = DailyData::new(date(22), &Timesheet::default(), &settings(), at(22, 12, 0))?;

        assert_eq!("00:00", day.overtime().to_string());
        Ok(())
    }

    #[test]
    fn should_end_a_running_day_at_now() -> Result<(), TrackerError> {
        let timesheet = timesheet(19, &[(Status::Connect, 8, 0)]);
        let day = DailyData::new(date(19), &timesheet, &settings(), at(19, 10, 0))?;

        assert!(day.end().is_none());
        // Two hours online with the expected break of 30 minutes.
        assert_eq!("01:30", day.worktime().to_string());
        Ok(())
    }

    #[test]
    fn should_reject_a_day_without_connect() {
        let timesheet = timesheet(17, &[(Status::End, 17, 0)]);

        assert!(DailyData::new(date(17), &timesheet, &settings(), at(21, 12, 0)).is_err());
    }
}
