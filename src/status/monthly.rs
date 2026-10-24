use std::fmt::Display;

use chrono::{ DateTime, FixedOffset, Month, NaiveDate };
use prettytable::{ format, Table };

use crate::{ Settings, StatusDisplay, StatusTime, MonthlyData, TrackerError, Weekly };

/// Status of a month, made of the weekly status of each of its weeks.
#[derive(Clone, Debug)]
pub struct Monthly {
    year: i32,
    month: u32,
    pub(crate) total: StatusTime,
    pub(crate) overtime: StatusTime,
    decimal: f64,
    weeks: Vec<Weekly>,
    /// Overtime accumulated before the month.
    carry: Option<StatusTime>,
}

impl Monthly {
    /// Status of `data` at `now`, which ends a day that is still running.
    pub fn new(
        data: &MonthlyData,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<Self, TrackerError> {
        let mut total = StatusTime::default();
        let mut overtime = StatusTime::default();
        let mut weeks = Vec::new();
        for week in &data.weeks {
            let s = Weekly::new(week, settings, now)?;
            total += s.total.to_owned();
            overtime += s.overtime.to_owned();
            weeks.push(s);
        }
        let decimal = (total.duration.num_minutes() as f64) / 60.0;

        Ok(Monthly {
            year: data.year,
            month: data.month,
            total,
            overtime,
            decimal,
            weeks,
            carry: None,
        })
    }

    /// Show the overtime accumulated before the month in the table.
    pub fn with_carry(mut self, carry: Option<StatusTime>) -> Self {
        self.carry = carry;
        self
    }

    pub(crate) fn month_name(&self) -> &'static str {
        Month::try_from(self.month as u8).map_or("", |m| m.name())
    }

    fn name(&self) -> String {
        format!("{} {}", self.month_name(), self.year)
    }

    pub fn start(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, self.month, 1).unwrap()
    }
}

impl StatusDisplay for Monthly {
    fn table(&self) -> String {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_COLSEP);
        table.set_titles(row!["Week", "Work time", "Overtime", "Cumulated"]);

        if let Some(carry) = &self.carry {
            table.add_row(row!["Carried over", "", "", carry]);
        }
        let cumulated = StatusTime::cumulated(
            self.carry.to_owned().unwrap_or_default(),
            self.weeks.iter().map(|w| &w.overtime)
        );
        for (week, cumulated) in self.weeks.iter().zip(cumulated) {
            table.add_row(row![week.week, week.total, week.overtime, cumulated]);
        }
        table.add_empty_row();
        table.add_row(row![format!("Total {}", self.name()), self.total, self.overtime, ""]);

        table.to_string()
    }
}

impl Display for Monthly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = 14;
        writeln!(f, " {:width$} | {:>10} | {:>10} | {:>10}", "Month", "Work time", "Overtime", "Decimal")?;
        writeln!(f, " {0:->width$} | {0:->10} | {0:->10} | {0:->10}", "")?;
        writeln!(
            f,
            " {:width$} | {:>10} | {:>10} | {:>10.2}",
            self.name(),
            self.total.to_string(),
            self.overtime.to_string(),
            self.decimal
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ DateTime, FixedOffset, NaiveDate };

    use crate::{
        MemoryStorage,
        Settings,
        Status,
        StatusDisplay,
        StatusTime,
        MonthlyData,
        TrackerError,
    };

    use super::*;

    /// All days are ended or missing, so the clock doesn't matter.
    fn now() -> DateTime<FixedOffset> {
        DateTime::default()
    }

    /// March 2022 with 8 hours on each workday of week 10 (7th to 11th) and nothing else.
    fn march() -> Result<Monthly, TrackerError> {
        let storage = (7..=11).fold(MemoryStorage::default(), |storage, day| {
            storage.day(
                NaiveDate::from_ymd_opt(2022, 3, day).unwrap(),
                &[(Status::Connect, 6, 0), (Status::End, 14, 0)]
            )
        });
        let data = MonthlyData::for_month(
            &storage,
            3,
            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap()
        )?;
        Monthly::new(&data, &Settings::default(), now())
    }

    #[test]
    fn should_sum_weeks_of_month() -> Result<(), TrackerError> {
        let s = march()?;

        let rows: Vec<(i16, String, String)> = s.weeks
            .iter()
            .map(|w| (w.week, w.total.to_string(), w.overtime.to_string()))
            .collect();
        assert_eq!(
            vec![
                // Tuesday to Sunday, 4 workdays missing.
                (9, "00:00".to_owned(), "-32:00".to_owned()),
                (10, "40:00".to_owned(), "00:00".to_owned()),
                (11, "00:00".to_owned(), "-40:00".to_owned()),
                (12, "00:00".to_owned(), "-40:00".to_owned()),
                // Monday to Thursday.
                (13, "00:00".to_owned(), "-32:00".to_owned())
            ],
            rows
        );
        assert_eq!("40:00", s.total.to_string());
        assert_eq!("-144:00", s.overtime.to_string());
        assert_eq!(40.0, s.decimal);
        Ok(())
    }

    #[test]
    fn should_display_month() -> Result<(), TrackerError> {
        let s = march()?;

        let out = s.to_string();
        let last = out.lines().last().unwrap();
        assert!(last.contains("March 2022"), "{out}");
        assert!(last.contains("40:00"), "{out}");
        assert!(last.contains("-144:00"), "{out}");
        assert!(last.contains("40.00"), "{out}");
        Ok(())
    }

    #[test]
    fn should_name_and_start_with_the_month() -> Result<(), TrackerError> {
        let s = march()?;

        assert_eq!("March", s.month_name());
        assert_eq!(NaiveDate::from_ymd_opt(2022, 3, 1).unwrap(), s.start());
        Ok(())
    }

    mod table {
        use super::*;

        #[test]
        fn should_show_a_row_for_each_week_and_the_total() -> Result<(), TrackerError> {
            let table = march()?.table();

            for week in ["-32:00", "40:00", "-40:00", "-144:00", "Total March 2022"] {
                assert!(table.contains(week), "{week} in {table}");
            }
            assert!(!table.contains("Carried over"), "{table}");
            Ok(())
        }

        #[test]
        fn should_add_the_carry_to_the_table_and_the_cumulated_overtime() -> Result<(), TrackerError> {
            let carry = StatusTime::from(chrono::Duration::hours(10));
            let table = march()?.with_carry(Some(carry)).table();

            let carried = table.lines().find(|l| l.contains("Carried over")).unwrap();
            assert!(carried.contains("10:00"), "{carried}");
            // The first week misses 32 hours, so the cumulated overtime is 10 - 32.
            let first = table.lines().find(|l| l.trim_start().starts_with("9 ")).unwrap();
            assert!(first.trim_end().ends_with("-22:00"), "{first}");
            Ok(())
        }

        #[test]
        fn should_remove_the_carry() -> Result<(), TrackerError> {
            let carry = Some(StatusTime::from(chrono::Duration::hours(10)));
            let s = march()?.with_carry(carry).with_carry(None);

            assert!(!s.table().contains("Carried over"));
            Ok(())
        }
    }
}
