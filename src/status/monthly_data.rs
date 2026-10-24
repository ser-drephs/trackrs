use chrono::{ Datelike, NaiveDate };

use crate::{ StorageProvider, WeeklyData, TrackerError };

/// Time data of a month, split into its ISO weeks. The first and last week only hold the days
/// within the month.
#[derive(Default, Clone)]
pub struct MonthlyData {
    pub year: i32,
    pub month: u32,
    pub weeks: Vec<WeeklyData>,
}

impl MonthlyData {
    /// Load `month` (1-12), or a month relative to the one of `today` for 0 and less, e.g. -1.
    pub fn for_month(
        storage: &impl StorageProvider,
        month: i16,
        today: NaiveDate
    ) -> Result<Self, TrackerError> {
        let (year, month) = Self::resolve(month, today)?;
        Self::for_year_month(storage, year, month)
    }

    /// Load `month` (1-12) of `year`.
    pub fn for_year_month(
        storage: &impl StorageProvider,
        year: i32,
        month: u32
    ) -> Result<Self, TrackerError> {
        let days: Vec<NaiveDate> = NaiveDate::from_ymd_opt(year, month, 1)
            .unwrap()
            .iter_days()
            .take_while(|d| d.month() == month)
            .collect();

        let mut weeks = Vec::new();
        for days in days.chunk_by(|a, b| a.iso_week() == b.iso_week()) {
            let week = days[0].iso_week().week().try_into()?;
            weeks.push(WeeklyData::for_dates(storage, week, days.to_vec())?);
        }

        Ok(MonthlyData { year, month, weeks })
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

    use crate::{ MemoryStorage, Status, MonthlyData, TrackerError };

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    /// The week numbers of the weeks of the month with their number of days.
    fn weeks(m: &MonthlyData) -> Vec<(i16, usize)> {
        m.weeks
            .iter()
            .map(|w| (w.week, w.days.len()))
            .collect()
    }

    mod resolve {
        use super::*;

        #[test]
        fn should_resolve_absolute_and_relative_months() {
            let today = date(2022, 2, 15);
            assert_eq!((2022, 5), MonthlyData::resolve(5, today).unwrap());
            assert_eq!((2022, 2), MonthlyData::resolve(0, today).unwrap());
            assert_eq!((2022, 1), MonthlyData::resolve(-1, today).unwrap());
            assert_eq!((2021, 12), MonthlyData::resolve(-2, today).unwrap());
            assert_eq!((2020, 12), MonthlyData::resolve(-14, today).unwrap());
            assert!(MonthlyData::resolve(13, today).is_err());
        }
    }

    mod for_month {
        use super::*;

        #[test]
        fn should_split_month_into_weeks() -> Result<(), TrackerError> {
            let m = MonthlyData::for_month(&MemoryStorage::default(), 3, date(2022, 6, 1))?;

            // March 2022 starts on a Tuesday and ends on a Thursday.
            assert_eq!(vec![(9, 6), (10, 7), (11, 7), (12, 7), (13, 4)], weeks(&m));
            assert_eq!((2022, 3), (m.year, m.month));
            Ok(())
        }

        #[test]
        fn should_count_back_from_the_month_of_today() -> Result<(), TrackerError> {
            let m = MonthlyData::for_month(&MemoryStorage::default(), -1, date(2022, 6, 1))?;

            // May 2022 starts on a Sunday and ends on a Tuesday.
            assert_eq!((2022, 5), (m.year, m.month));
            assert_eq!(vec![(17, 1), (18, 7), (19, 7), (20, 7), (21, 7), (22, 2)], weeks(&m));
            Ok(())
        }

        #[test]
        fn should_reject_month_not_in_year() {
            let err = MonthlyData::for_month(&MemoryStorage::default(), 13, date(2022, 6, 1));

            assert_eq!("time data error: month 13 doesn't exist", err.err().unwrap().to_string());
        }
    }

    mod for_year_month {
        use super::*;

        #[test]
        fn should_load_the_timesheets_of_the_month() -> Result<(), TrackerError> {
            let storage = MemoryStorage::default()
                .day(date(2022, 3, 7), &[(Status::Connect, 8, 0), (Status::End, 16, 0)])
                .day(date(2022, 3, 31), &[(Status::Connect, 9, 0)])
                // Not in March, even though it is in the same week as the 31st.
                .day(date(2022, 4, 1), &[(Status::Connect, 9, 0)]);
            let m = MonthlyData::for_year_month(&storage, 2022, 3)?;

            let entries: usize = m.weeks
                .iter()
                .flat_map(|w| &w.days)
                .map(|(_, t)| t.data.len())
                .sum();
            assert_eq!(3, entries);
            let last_week = m.weeks.last().unwrap();
            assert_eq!(date(2022, 3, 28), last_week.days.first().unwrap().0);
            assert_eq!(date(2022, 3, 31), last_week.days.last().unwrap().0);
            Ok(())
        }

        #[test]
        fn should_have_the_days_of_each_month_once() -> Result<(), TrackerError> {
            let days: usize = (1..=12)
                .map(|month| MonthlyData::for_year_month(&MemoryStorage::default(), 2022, month))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .flat_map(|m| &m.weeks)
                .map(|w| w.days.len())
                .sum();

            assert_eq!(365, days);
            Ok(())
        }
    }
}
