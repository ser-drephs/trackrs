use std::fmt::Display;

use chrono::{ Month, NaiveDate };
use prettytable::{ format, Table };

use crate::{ Settings, StatusTime, StatusWeekly, TimeDataMonthly, TrackerError };

/// Status of a month, made of the weekly status of each of its weeks.
#[derive(Clone, Debug)]
pub struct StatusMonthly {
    year: i32,
    month: u32,
    pub(crate) total: StatusTime,
    pub(crate) overtime: StatusTime,
    decimal: f64,
    weeks: Vec<StatusWeekly>,
}

impl StatusMonthly {
    pub fn new(data: TimeDataMonthly, settings: &Settings) -> Result<Self, TrackerError> {
        let mut total = StatusTime::default();
        let mut overtime = StatusTime::default();
        let mut weeks = Vec::new();
        for week in data.weeks {
            let s = StatusWeekly::builder().data(week).settings(settings.to_owned()).build()?;
            total += s.total.to_owned();
            overtime += s.overtime.to_owned();
            weeks.push(s);
        }
        let decimal = (total.duration.num_minutes() as f64) / 60.0;

        Ok(StatusMonthly { year: data.year, month: data.month, total, overtime, decimal, weeks })
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

    /// Print the table; `carry` is the overtime accumulated before the month.
    pub fn format_table(&self, carry: Option<StatusTime>) {
        let mut table = Table::new();
        table.set_format(*format::consts::FORMAT_NO_COLSEP);
        table.set_titles(row!["Week", "Work time", "Overtime", "Cumulated"]);

        if let Some(carry) = &carry {
            table.add_row(row!["Carried over", "", "", carry]);
        }
        let cumulated = StatusTime::cumulated(
            carry.unwrap_or_default(),
            self.weeks.iter().map(|w| &w.overtime)
        );
        for (week, cumulated) in self.weeks.iter().zip(cumulated) {
            table.add_row(row![week.week, week.total, week.overtime, cumulated]);
        }
        table.add_empty_row();
        table.add_row(row![format!("Total {}", self.name()), self.total, self.overtime, ""]);

        table.printstd();
    }
}

impl Display for StatusMonthly {
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
    use std::fs;

    use chrono::NaiveDate;

    use crate::{ Settings, StatusMonthly, TimeDataMonthly, TrackerError };

    /// March 2022 with 8 hours on each workday of week 10 (7th to 11th) and nothing else.
    fn march(folder: &std::path::Path) -> Result<StatusMonthly, TrackerError> {
        for day in 7..=11 {
            let content = format!(
                "{{\"data\":[{{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-03-{day:02}T06:00:00Z\"}},{{\"id\":2,\"status\":\"End\",\"time\":\"2022-03-{day:02}T14:00:00Z\"}}],\"version\":2}}"
            );
            fs::write(folder.join(format!("202203{day:02}.json")), content)?;
        }
        let data = TimeDataMonthly::load(
            folder.to_path_buf().into(),
            3,
            NaiveDate::from_ymd_opt(2022, 6, 1).unwrap()
        )?;
        StatusMonthly::new(data, &Settings::default())
    }

    #[test]
    fn should_sum_weeks_of_month() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        let s = march(temp_dir.path())?;

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
        let temp_dir = tempfile::tempdir()?;
        let s = march(temp_dir.path())?;

        let out = s.to_string();
        let last = out.lines().last().unwrap();
        assert!(last.contains("March 2022"), "{out}");
        assert!(last.contains("40:00"), "{out}");
        assert!(last.contains("-144:00"), "{out}");
        assert!(last.contains("40.00"), "{out}");
        Ok(())
    }
}
