use chrono::{ Datelike, NaiveDate };

use crate::{ StorageProvider, MonthlyData, TrackerError };

/// Time data of a year, split into its months.
#[derive(Default, Clone)]
pub struct YearlyData {
    pub year: i32,
    pub months: Vec<MonthlyData>,
}

impl YearlyData {
    /// Load `year`, or a year relative to the one of `today` for 0 and less, e.g. -1.
    pub fn for_year(
        storage: &impl StorageProvider,
        year: i16,
        today: NaiveDate
    ) -> Result<Self, TrackerError> {
        let year = Self::resolve(year, today);
        let months = (1..=12)
            .map(|month| MonthlyData::for_year_month(storage, year, month))
            .collect::<Result<_, _>>()?;
        Ok(YearlyData { year, months })
    }

    fn resolve(year: i16, today: NaiveDate) -> i32 {
        let year = i32::from(year);
        if year > 0 { year } else { today.year() + year }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{ MemoryStorage, Status, YearlyData, TrackerError };

    fn today() -> NaiveDate {
        NaiveDate::from_ymd_opt(2026, 10, 24).unwrap()
    }

    /// The number of days of all months of the year.
    fn days(y: &YearlyData) -> usize {
        y.months
            .iter()
            .flat_map(|m| &m.weeks)
            .map(|w| w.days.len())
            .sum()
    }

    mod resolve {
        use super::*;

        #[test]
        fn should_resolve_absolute_and_relative_years() {
            assert_eq!(2022, YearlyData::resolve(2022, today()));
            assert_eq!(2026, YearlyData::resolve(0, today()));
            assert_eq!(2025, YearlyData::resolve(-1, today()));
        }
    }

    mod for_year {
        use super::*;

        #[test]
        fn should_have_all_months() -> Result<(), TrackerError> {
            let y = YearlyData::for_year(&MemoryStorage::default(), 2022, today())?;

            let months: Vec<u32> = y.months.iter().map(|m| m.month).collect();
            assert_eq!((1..=12).collect::<Vec<u32>>(), months);
            assert_eq!(365, days(&y));
            assert_eq!(2022, y.year);
            Ok(())
        }

        #[test]
        fn should_have_the_days_of_a_leap_year() -> Result<(), TrackerError> {
            let y = YearlyData::for_year(&MemoryStorage::default(), 2024, today())?;

            assert_eq!(366, days(&y));
            Ok(())
        }

        #[test]
        fn should_count_back_from_the_year_of_today() -> Result<(), TrackerError> {
            let y = YearlyData::for_year(&MemoryStorage::default(), -1, today())?;

            assert_eq!(2025, y.year);
            Ok(())
        }

        #[test]
        fn should_load_the_timesheets_of_the_year() -> Result<(), TrackerError> {
            let storage = MemoryStorage::default()
                .day(NaiveDate::from_ymd_opt(2022, 1, 3).unwrap(), &[(Status::Connect, 8, 0)])
                .day(
                    NaiveDate::from_ymd_opt(2022, 12, 30).unwrap(),
                    &[(Status::Connect, 8, 0), (Status::End, 16, 0)]
                )
                // Another year.
                .day(NaiveDate::from_ymd_opt(2023, 1, 2).unwrap(), &[(Status::Connect, 8, 0)]);
            let y = YearlyData::for_year(&storage, 2022, today())?;

            let entries: usize = y.months
                .iter()
                .flat_map(|m| &m.weeks)
                .flat_map(|w| &w.days)
                .map(|(_, t)| t.data.len())
                .sum();
            assert_eq!(3, entries);
            Ok(())
        }
    }
}
