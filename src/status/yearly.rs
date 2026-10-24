use std::fmt::Display;

use chrono::{ DateTime, FixedOffset, NaiveDate };
use prettytable::{ format, Table };

use crate::{ Monthly, Settings, StatusDisplay, StatusTime, YearlyData, TrackerError };

/// Status of a year, made of the monthly status of each of its months.
#[derive(Clone, Debug)]
pub struct Yearly {
    year: i32,
    total: StatusTime,
    overtime: StatusTime,
    decimal: f64,
    months: Vec<Monthly>,
    /// Overtime accumulated before the year.
    carry: Option<StatusTime>,
}

impl Yearly {
    /// Status of `data` at `now`, which ends a day that is still running.
    pub fn new(
        data: &YearlyData,
        settings: &Settings,
        now: DateTime<FixedOffset>
    ) -> Result<Self, TrackerError> {
        let mut total = StatusTime::default();
        let mut overtime = StatusTime::default();
        let mut months = Vec::new();
        for month in &data.months {
            let s = Monthly::new(month, settings, now)?;
            total += s.total.to_owned();
            overtime += s.overtime.to_owned();
            months.push(s);
        }
        let decimal = (total.duration.num_minutes() as f64) / 60.0;

        Ok(Yearly { year: data.year, total, overtime, decimal, months, carry: None })
    }

    /// Show the overtime accumulated before the year in the table.
    pub fn with_carry(mut self, carry: Option<StatusTime>) -> Self {
        self.carry = carry;
        self
    }

    pub fn start(&self) -> NaiveDate {
        NaiveDate::from_ymd_opt(self.year, 1, 1).unwrap()
    }
}

impl StatusDisplay for Yearly {
    fn table(&self) -> String {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_COLSEP);
        table.set_titles(row!["Month", "Work time", "Overtime", "Cumulated"]);

        if let Some(carry) = &self.carry {
            table.add_row(row!["Carried over", "", "", carry]);
        }
        let cumulated = StatusTime::cumulated(
            self.carry.to_owned().unwrap_or_default(),
            self.months.iter().map(|m| &m.overtime)
        );
        for (month, cumulated) in self.months.iter().zip(cumulated) {
            table.add_row(row![month.month_name(), month.total, month.overtime, cumulated]);
        }
        table.add_empty_row();
        table.add_row(row![format!("Total {}", self.year), self.total, self.overtime, ""]);

        table.to_string()
    }
}

impl Display for Yearly {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let width = 10;
        writeln!(f, " {:width$} | {:>width$} | {:>width$} | {:>width$}", "Year", "Work time", "Overtime", "Decimal")?;
        writeln!(f, " {0:->width$} | {0:->width$} | {0:->width$} | {0:->width$}", "")?;
        writeln!(
            f,
            " {:width$} | {:>width$} | {:>width$} | {:>width$.2}",
            self.year,
            self.total.to_string(),
            self.overtime.to_string(),
            self.decimal
        )
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;

    use crate::{
        MemoryStorage,
        Settings,
        Status,
        StatusDisplay,
        StatusTime,
        YearlyData,
        TrackerError,
    };

    use super::*;

    /// 2022 with 9 hours on each workday of week 10 (7th to 11th March) and nothing else.
    fn year() -> Result<Yearly, TrackerError> {
        let storage = (7..=11).fold(MemoryStorage::default(), |storage, day| {
            storage.day(
                NaiveDate::from_ymd_opt(2022, 3, day).unwrap(),
                &[(Status::Connect, 6, 0), (Status::End, 15, 0)]
            )
        });
        let today = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        let data = YearlyData::for_year(&storage, 2022, today)?;
        // All days are ended or missing, so the clock doesn't matter.
        Yearly::new(&data, &Settings::default(), DateTime::default())
    }

    #[test]
    fn should_sum_months_of_year() -> Result<(), TrackerError> {
        let s = year()?;

        let rows: Vec<(&str, String, String)> = s.months
            .iter()
            .map(|m| (m.month_name(), m.total.to_string(), m.overtime.to_string()))
            .collect();
        // January 2022 has 21 workdays, March 23 of which 5 were worked with an hour extra.
        assert_eq!(("January", "00:00".to_owned(), "-168:00".to_owned()), rows[0]);
        assert_eq!(("March", "45:00".to_owned(), "-139:00".to_owned()), rows[2]);
        assert_eq!(12, rows.len());

        // 2022 has 260 workdays of 8 hours.
        assert_eq!("45:00", s.total.to_string());
        assert_eq!("-2035:00", s.overtime.to_string());
        assert_eq!(45.0, s.decimal);
        Ok(())
    }

    #[test]
    fn should_display_year() -> Result<(), TrackerError> {
        let out = year()?.to_string();
        let last = out.lines().last().unwrap();

        assert!(last.contains("2022"), "{out}");
        assert!(last.contains("45:00"), "{out}");
        assert!(last.contains("-2035:00"), "{out}");
        assert!(last.contains("45.00"), "{out}");
        Ok(())
    }

    #[test]
    fn should_start_on_the_first_of_january() -> Result<(), TrackerError> {
        assert_eq!(NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(), year()?.start());
        Ok(())
    }

    mod table {
        use super::*;

        #[test]
        fn should_show_a_row_for_each_month_and_the_total() -> Result<(), TrackerError> {
            let table = year()?.table();

            for month in [
                "January",
                "February",
                "March",
                "April",
                "May",
                "June",
                "July",
                "August",
                "September",
                "October",
                "November",
                "December",
            ] {
                assert!(table.contains(month), "{month} in {table}");
            }
            assert!(table.contains("Total 2022"), "{table}");
            assert!(table.contains("-2035:00"), "{table}");
            assert!(!table.contains("Carried over"), "{table}");
            Ok(())
        }

        #[test]
        fn should_add_the_carry_to_the_table_and_the_cumulated_overtime() -> Result<(), TrackerError> {
            let carry = StatusTime::from(chrono::Duration::hours(10));
            let table = year()?.with_carry(Some(carry)).table();

            let carried = table.lines().find(|l| l.contains("Carried over")).unwrap();
            assert!(carried.contains("10:00"), "{carried}");
            // January misses 168 hours, so the cumulated overtime is 10 - 168.
            let january = table.lines().find(|l| l.contains("January")).unwrap();
            assert!(january.trim_end().ends_with("-158:00"), "{january}");
            Ok(())
        }

        #[test]
        fn should_remove_the_carry() -> Result<(), TrackerError> {
            let carry = Some(StatusTime::from(chrono::Duration::hours(10)));
            let s = year()?.with_carry(carry).with_carry(None);

            assert!(!s.table().contains("Carried over"));
            Ok(())
        }
    }
}
