use chrono::{ Datelike, NaiveDate };

use crate::{ Folder, TimeDataWeekly, TrackerError };

/// Time data of a month, split into its ISO weeks. The first and last week only hold the days
/// within the month.
#[derive(Default, Clone)]
pub struct TimeDataMonthly {
    pub year: i32,
    pub month: u32,
    pub weeks: Vec<TimeDataWeekly>,
}

impl TimeDataMonthly {
    /// Load `month` (1-12), or a month relative to the one of `today` for 0 and less, e.g. -1.
    pub fn load(folder: Folder, month: i16, today: NaiveDate) -> Result<Self, TrackerError> {
        let (year, month) = Self::resolve(month, today)?;
        Self::load_month(folder, year, month)
    }

    /// Load `month` (1-12) of `year`.
    pub fn load_month(folder: Folder, year: i32, month: u32) -> Result<Self, TrackerError> {
        let days: Vec<NaiveDate> = NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap()
            .iter_days()
            .take_while(|d| d.month() == month)
            .collect();

        let mut weeks = Vec::new();
        for days in days.chunk_by(|a, b| a.iso_week() == b.iso_week()) {
            let week = days[0].iso_week().week().try_into()?;
            let dates = days.to_vec();
            weeks.push(
                TimeDataWeekly::builder().folder(folder.to_owned()).week_dates(week, dates).build()?
            );
        }

        Ok(TimeDataMonthly { year, month, weeks })
    }

    pub(crate) fn resolve(month: i16, today: NaiveDate) -> Result<(i32, u32), TrackerError> {
        match month {
            1..=12 => Ok((today.year(), month as u32)),
            13.. => Err(TrackerError::TimeDataError {
                message: format!("month {month} doesn't exist"),
            }),
            _ => {
                let index = today.year() * 12 + (today.month0() as i32) + (month as i32);
                Ok((index.div_euclid(12), (index.rem_euclid(12) + 1) as u32))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{ TimeDataMonthly, TrackerError };

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn should_resolve_absolute_and_relative_months() {
        let today = date(2022, 2, 15);
        assert_eq!((2022, 5), TimeDataMonthly::resolve(5, today).unwrap());
        assert_eq!((2022, 2), TimeDataMonthly::resolve(0, today).unwrap());
        assert_eq!((2022, 1), TimeDataMonthly::resolve(-1, today).unwrap());
        assert_eq!((2021, 12), TimeDataMonthly::resolve(-2, today).unwrap());
        assert_eq!((2020, 12), TimeDataMonthly::resolve(-14, today).unwrap());
        assert!(TimeDataMonthly::resolve(13, today).is_err());
    }

    #[test]
    fn should_split_month_into_weeks() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        let m = TimeDataMonthly::load(temp_dir.path().to_path_buf().into(), 3, date(2022, 6, 1))?;

        // March 2022 starts on a Tuesday and ends on a Thursday.
        let weeks: Vec<(i16, usize)> = m.weeks
            .iter()
            .map(|w| (w.week, w.dates.len()))
            .collect();
        assert_eq!(vec![(9, 6), (10, 7), (11, 7), (12, 7), (13, 4)], weeks);
        assert_eq!((2022, 3), (m.year, m.month));
        Ok(())
    }
}
