use chrono::{ Datelike, NaiveDate };

use crate::{ StorageProvider, TimeDataMonthly, TrackerError };

/// Time data of a year, split into its months.
#[derive(Default, Clone)]
pub struct TimeDataYearly {
    pub year: i32,
    pub months: Vec<TimeDataMonthly>,
}

impl TimeDataYearly {
    /// Load `year`, or a year relative to the one of `today` for 0 and less, e.g. -1.
    pub fn load(
        storage: &impl StorageProvider,
        year: i16,
        today: NaiveDate
    ) -> Result<Self, TrackerError> {
        let year = Self::resolve(year, today);
        let months = (1..=12)
            .map(|month| TimeDataMonthly::load_month(storage, year, month))
            .collect::<Result<_, _>>()?;
        Ok(TimeDataYearly { year, months })
    }

    fn resolve(year: i16, today: NaiveDate) -> i32 {
        let year = i32::from(year);
        if year > 0 { year } else { today.year() + year }
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{ MemoryStorage, TimeDataYearly, TrackerError };

    #[test]
    fn should_resolve_absolute_and_relative_years() {
        let today = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        assert_eq!(2022, TimeDataYearly::resolve(2022, today));
        assert_eq!(2026, TimeDataYearly::resolve(0, today));
        assert_eq!(2025, TimeDataYearly::resolve(-1, today));
    }

    #[test]
    fn should_load_all_months() -> Result<(), TrackerError> {
        let today = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        let y = TimeDataYearly::load(&MemoryStorage::default(), 2022, today)?;

        let months: Vec<u32> = y.months.iter().map(|m| m.month).collect();
        assert_eq!((1..=12).collect::<Vec<u32>>(), months);
        let days: usize = y.months
            .iter()
            .flat_map(|m| &m.weeks)
            .map(|w| w.days.len())
            .sum();
        assert_eq!(365, days);
        assert_eq!(2022, y.year);
        Ok(())
    }
}
