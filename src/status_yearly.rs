use std::fmt::Display;

use prettytable::{ format, Table };

use crate::{ Settings, StatusMonthly, StatusTime, TimeDataYearly, TrackerError };

/// Status of a year, made of the monthly status of each of its months.
#[derive(Clone, Debug)]
pub struct StatusYearly {
    year: i32,
    total: StatusTime,
    overtime: StatusTime,
    decimal: f64,
    months: Vec<StatusMonthly>,
}

impl StatusYearly {
    pub fn new(data: TimeDataYearly, settings: &Settings) -> Result<Self, TrackerError> {
        let mut total = StatusTime::default();
        let mut overtime = StatusTime::default();
        let mut months = Vec::new();
        for month in data.months {
            let s = StatusMonthly::new(month, settings)?;
            total += s.total.to_owned();
            overtime += s.overtime.to_owned();
            months.push(s);
        }
        let decimal = (total.duration.num_minutes() as f64) / 60.0;

        Ok(StatusYearly { year: data.year, total, overtime, decimal, months })
    }

    pub fn format_table(&self) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_COLSEP);
        table.set_titles(row!["Month", "Work time", "Overtime", "Cumulated"]);

        let cumulated = StatusTime::cumulated(self.months.iter().map(|m| &m.overtime));
        for (month, cumulated) in self.months.iter().zip(cumulated) {
            table.add_row(row![month.month_name(), month.total, month.overtime, cumulated]);
        }
        table.add_empty_row();
        table.add_row(row![format!("Total {}", self.year), self.total, self.overtime, ""]);

        table.printstd();
    }
}

impl Display for StatusYearly {
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
    use std::fs;

    use chrono::NaiveDate;

    use crate::{ Settings, StatusYearly, TimeDataYearly, TrackerError };

    /// 2022 with 9 hours on each workday of week 10 (7th to 11th March) and nothing else.
    fn year(folder: &std::path::Path) -> Result<StatusYearly, TrackerError> {
        for day in 7..=11 {
            let content = format!(
                "{{\"data\":[{{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-03-{day:02}T06:00:00Z\"}},{{\"id\":2,\"status\":\"End\",\"time\":\"2022-03-{day:02}T15:00:00Z\"}}],\"version\":2}}"
            );
            fs::write(folder.join(format!("202203{day:02}.json")), content)?;
        }
        let today = NaiveDate::from_ymd_opt(2026, 10, 24).unwrap();
        let data = TimeDataYearly::load(folder.to_path_buf().into(), 2022, today)?;
        StatusYearly::new(data, &Settings::default())
    }

    #[test]
    fn should_sum_months_of_year() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        let s = year(temp_dir.path())?;

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
        let temp_dir = tempfile::tempdir()?;
        let out = year(temp_dir.path())?.to_string();
        let last = out.lines().last().unwrap();

        assert!(last.contains("2022"), "{out}");
        assert!(last.contains("45:00"), "{out}");
        assert!(last.contains("-2035:00"), "{out}");
        assert!(last.contains("45.00"), "{out}");
        Ok(())
    }
}
