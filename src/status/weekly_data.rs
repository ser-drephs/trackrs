use chrono::{ Datelike, DateTime, Duration, FixedOffset, IsoWeek, NaiveDate, Weekday };

use crate::{ DailyData, Settings, StorageProvider, Timesheet, TrackerError };

/// The days of a week, or of its days within a month, with their timesheets.
#[derive(Default, Clone)]
pub struct WeeklyData {
    pub week: i16,
    pub days: Vec<(NaiveDate, Timesheet)>,
}

impl WeeklyData {
    /// Load the ISO week `week` of `year`. A week of 0 or less counts back from `current_week`,
    /// e.g. -1 for last week, also across the new year.
    pub fn for_week(
        storage: &impl StorageProvider,
        year: u16,
        week: i16,
        current_week: IsoWeek
    ) -> Result<Self, TrackerError> {
        let (year, week) = resolve_week(year, week, current_week);
        Self::for_dates(storage, week, week_dates(year, week)?)
    }

    /// Load only the given `dates`, e.g. the days of a week within a month, as `week`.
    pub fn for_dates(
        storage: &impl StorageProvider,
        week: i16,
        dates: Vec<NaiveDate>
    ) -> Result<Self, TrackerError> {
        let days = dates
            .into_iter()
            .map(|date| Ok((date, storage.load(date)?)))
            .collect::<Result<_, TrackerError>>()?;
        Ok(WeeklyData { week, days })
    }

    /// First day of the week, or of its days that were loaded.
    pub fn start(&self) -> Option<NaiveDate> {
        self.days.first().map(|(date, _)| *date)
    }

    /// What each day adds up to at `now`, which ends a day that is still running.
    pub fn day_data(
        &self,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<Vec<(NaiveDate, DailyData)>, TrackerError> {
        self.days
            .iter()
            .map(|(date, timesheet)| {
                log::trace!("processing: {}", date);
                Ok((*date, DailyData::new(*date, timesheet, settings, now)?))
            })
            .collect()
    }
}

/// The week as counted back from `current_week` if it is 0 or less, moved back into the previous
/// years while it is before the first week of `year`.
fn resolve_week(year: u16, week: i16, current_week: IsoWeek) -> (u16, i16) {
    let mut week = if week > 0 {
        week
    } else {
        i16::try_from(current_week.week()).unwrap() + week
    };
    let mut year = year;
    while week < 1 {
        year -= 1;
        // 28th of December is always in the last week of its year.
        let weeks = NaiveDate::from_ymd_opt(year.into(), 12, 28).unwrap().iso_week().week();
        week += weeks as i16;
    }
    (year, week)
}

/// The seven days of the ISO week `week` of `year`, starting on Monday.
fn week_dates(year: u16, week: i16) -> Result<Vec<NaiveDate>, TrackerError> {
    let (year, week) = (i32::from(year), u32::try_from(week)?);
    let monday = NaiveDate::from_isoywd_opt(year, week, Weekday::Mon).ok_or_else(||
        TrackerError::TimeDataError {
            message: format!("week {week} doesn't exist in {year}"),
        }
    )?;
    let dates = (0..7).map(|day| monday + Duration::days(day)).collect();
    log::debug!("dates of week {} of {}: {:?}", week, year, dates);
    Ok(dates)
}

#[cfg(test)]
mod tests {
    use chrono::{ Datelike, TimeZone, Utc };

    use crate::{ MemoryStorage, Status };

    use super::*;

    fn logger() {
        // std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    /// Week 5 of 2022.
    fn current_week() -> IsoWeek {
        Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 0).unwrap().fixed_offset().iso_week()
    }

    /// Storage with entries on Monday the 17th and Wednesday the 19th of week 3 of 2022.
    fn storage() -> MemoryStorage {
        MemoryStorage::default()
            .day(
                NaiveDate::from_ymd_opt(2022, 1, 17).unwrap(),
                &[(Status::Connect, 8, 0), (Status::End, 16, 0)]
            )
            .day(NaiveDate::from_ymd_opt(2022, 1, 19).unwrap(), &[(Status::Connect, 9, 0)])
    }

    mod resolve_week {
        use super::*;

        #[test]
        fn should_keep_positive_week() {
            assert_eq!((2022, 2), resolve_week(2022, 2, current_week()));
        }

        #[test]
        fn should_count_back_from_current_week() {
            assert_eq!((2022, 3), resolve_week(2022, -2, current_week()));
        }

        #[test]
        fn should_resolve_zero_to_current_week() {
            assert_eq!((2022, 5), resolve_week(2022, 0, current_week()));
        }

        #[test]
        fn should_move_week_before_first_into_previous_year() {
            // Ten weeks back from week 5 is week -5, which is week 47 of 2021 with its 52 weeks.
            assert_eq!((2021, 47), resolve_week(2022, -10, current_week()));
        }

        #[test]
        fn should_move_back_across_several_years() {
            // Week -55 is week -3 of 2021 and week 50 of 2020 with its 53 weeks.
            assert_eq!((2020, 50), resolve_week(2022, -60, current_week()));
        }

        #[test]
        fn should_resolve_relative_week_zero_to_last_week_of_previous_year() {
            // Week 1 of 2021, one week back is week 53 of 2020.
            let d = Utc.with_ymd_and_hms(2021, 1, 6, 0, 0, 0).unwrap().fixed_offset();

            assert_eq!((2020, 53), resolve_week(2021, -1, d.iso_week()));
        }

        #[test]
        fn should_not_take_week_one_of_next_year_as_last_week() {
            // Week 2 of 2025, three weeks back is week 51 of 2024 with its 52 weeks, although
            // 31st of December 2024 is in week 1 of 2025.
            let d = Utc.with_ymd_and_hms(2025, 1, 8, 0, 0, 0).unwrap().fixed_offset();

            assert_eq!((2024, 51), resolve_week(2025, -3, d.iso_week()));
        }
    }

    mod week_dates {
        use super::*;

        #[test]
        fn should_list_the_seven_days_from_monday() -> Result<(), TrackerError> {
            logger();
            let dates = week_dates(2022, 3)?;

            assert_eq!(7, dates.len());
            assert_eq!(NaiveDate::from_ymd_opt(2022, 1, 17), dates.first().copied());
            assert_eq!(NaiveDate::from_ymd_opt(2022, 1, 23), dates.last().copied());
            Ok(())
        }

        #[test]
        fn should_list_days_of_last_week_of_previous_year() -> Result<(), TrackerError> {
            let (year, week) = resolve_week(2022, -60, current_week());
            let dates = week_dates(year, week)?;

            assert_eq!(7, dates.len());
            assert_eq!(7, dates.first().unwrap().day());
            assert_eq!(13, dates.last().unwrap().day());
            assert_eq!(12, dates.last().unwrap().month());
            assert_eq!(2020, dates.last().unwrap().year());
            Ok(())
        }

        #[test]
        fn should_reject_week_not_in_year() {
            let err = week_dates(2022, 60).err().unwrap();
            assert_eq!("time data error: week 60 doesn't exist in 2022", err.to_string());

            // 2022 has 52 weeks, 2020 has 53.
            assert!(week_dates(2022, 53).is_err());
            assert!(week_dates(2020, 53).is_ok());
        }

        #[test]
        fn should_reject_negative_week() {
            assert!(week_dates(2022, -1).is_err());
        }
    }

    mod for_week {
        use super::*;

        #[test]
        fn should_load_the_timesheets_of_the_week() -> Result<(), TrackerError> {
            logger();
            let t = WeeklyData::for_week(&storage(), 2022, -2, current_week())?;

            assert_eq!(3, t.week);
            assert_eq!(7, t.days.len());
            assert_eq!(NaiveDate::from_ymd_opt(2022, 1, 17), t.start());
            assert_eq!(NaiveDate::from_ymd_opt(2022, 1, 23), t.days.last().map(|(d, _)| *d));
            let entries: Vec<usize> = t.days.iter().map(|(_, ts)| ts.data.len()).collect();
            assert_eq!(vec![2, 0, 1, 0, 0, 0, 0], entries);
            Ok(())
        }

        #[test]
        fn should_load_the_given_week_of_the_year() -> Result<(), TrackerError> {
            let t = WeeklyData::for_week(&storage(), 2022, 3, current_week())?;

            assert_eq!(3, t.week);
            assert_eq!(NaiveDate::from_ymd_opt(2022, 1, 17), t.start());
            Ok(())
        }

        #[test]
        fn should_reject_week_not_in_year() {
            let err = WeeklyData::for_week(&storage(), 2022, 60, current_week()).err().unwrap();

            assert_eq!("time data error: week 60 doesn't exist in 2022", err.to_string());
        }
    }

    mod for_dates {
        use super::*;

        #[test]
        fn should_load_only_the_given_dates() -> Result<(), TrackerError> {
            let dates = vec![
                NaiveDate::from_ymd_opt(2022, 1, 19).unwrap(),
                NaiveDate::from_ymd_opt(2022, 1, 20).unwrap()
            ];
            let t = WeeklyData::for_dates(&storage(), 3, dates)?;

            assert_eq!(3, t.week);
            assert_eq!(2, t.days.len());
            assert_eq!(NaiveDate::from_ymd_opt(2022, 1, 19), t.start());
            assert_eq!(1, t.days[0].1.data.len());
            assert!(t.days[1].1.data.is_empty());
            Ok(())
        }

        #[test]
        fn should_have_no_start_without_dates() -> Result<(), TrackerError> {
            let t = WeeklyData::for_dates(&storage(), 3, Vec::new())?;

            assert!(t.days.is_empty());
            assert_eq!(None, t.start());
            Ok(())
        }
    }

    mod day_data {

        use crate::Entry;

        use super::*;

        fn now() -> DateTime<FixedOffset> {
            Utc.with_ymd_and_hms(2022, 1, 21, 12, 0, 0).unwrap().fixed_offset()
        }

        /// The days `days` of January 2022 without entries.
        fn week(days: &[u32]) -> WeeklyData {
            WeeklyData {
                week: 3,
                days: days
                    .iter()
                    .map(|d| (NaiveDate::from_ymd_opt(2022, 1, *d).unwrap(), Timesheet::default()))
                    .collect(),
            }
        }

        #[test]
        fn should_have_a_day_for_each_day_in_order() -> Result<(), TrackerError> {
            let days = week(&[19, 17, 18]).day_data(&Settings::default(), now())?;

            let dates: Vec<u32> = days.iter().map(|(date, _)| date.day()).collect();
            assert_eq!(vec![19, 17, 18], dates);
            Ok(())
        }

        #[test]
        fn should_have_no_days_without_days() -> Result<(), TrackerError> {
            assert!(week(&[]).day_data(&Settings::default(), now())?.is_empty());
            Ok(())
        }

        #[test]
        fn should_reject_a_day_without_connect() {
            let mut data = week(&[17]);
            data.days[0].1.data.push(Entry {
                status: Status::End,
                time: Utc.with_ymd_and_hms(2022, 1, 17, 17, 0, 0).unwrap().fixed_offset(),
                note: None,
            });

            assert!(data.day_data(&Settings::default(), now()).is_err());
        }
    }
}
