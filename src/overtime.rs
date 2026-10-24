use std::{
    collections::BTreeMap,
    fs,
    path::{ Path, PathBuf },
};

use chrono::{ Datelike, Months, NaiveDate };
use serde::{ Deserialize, Serialize };

use crate::{ Folder, Settings, StatusTime, StatusWeekly, TimeDataWeekly, TrackerError };

/// Overtime of a month and the overtime accumulated up to and including it, as `HH:mm`.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct MonthOvertime {
    pub overtime: String,
    pub cumulated: String,
}

/// Months by `YYYY-MM`, in order.
type Entries = BTreeMap<String, MonthOvertime>;

/// Stored overtime per month in `overtime.json` of the time data folder.
///
/// A month's cumulated overtime is the cumulated overtime of the stored month before it plus its
/// own overtime. The first stored month is free, so it can carry overtime from before.
pub struct Overtime {
    file: PathBuf,
}

fn key(year: i32, month: u32) -> String {
    format!("{year:04}-{month:02}")
}

fn error(message: String) -> TrackerError {
    TrackerError::OvertimeError { message }
}

fn parse(key: &str, value: &str) -> Result<StatusTime, TrackerError> {
    value.parse().map_err(|e| error(format!("{key}: {e}")))
}

impl Overtime {
    pub fn new(folder: &Path) -> Self {
        Overtime {
            file: folder.join("overtime.json"),
        }
    }

    fn read(&self) -> Result<Entries, TrackerError> {
        if !self.file.exists() {
            return Ok(Entries::new());
        }
        let entries: Entries = serde_json::from_str(&fs::read_to_string(&self.file)?)?;
        for (k, e) in &entries {
            NaiveDate::parse_from_str(&format!("{k}-01"), "%Y-%m-%d").map_err(|_|
                error(format!("'{k}' is not a month like 2025-01"))
            )?;
            parse(k, &e.overtime)?;
            parse(k, &e.cumulated)?;
        }
        Ok(entries)
    }

    /// Store the overtime of a finished month, keeping the cumulated overtime of all stored
    /// months consistent.
    pub fn store(
        &self,
        year: i32,
        month: u32,
        overtime: StatusTime,
        force: bool
    ) -> Result<MonthOvertime, TrackerError> {
        let mut entries = self.read()?;
        let stored = Self::insert(&mut entries, &key(year, month), overtime, force)?;
        fs::write(&self.file, serde_json::to_string_pretty(&entries)? + "\n")?;
        Ok(stored)
    }

    fn insert(
        entries: &mut Entries,
        key: &str,
        overtime: StatusTime,
        force: bool
    ) -> Result<MonthOvertime, TrackerError> {
        if !force {
            if entries.contains_key(key) {
                return Err(error(format!("{key} is already stored, use --force to replace it")));
            }
            if let Some(first) = entries.keys().next().filter(|f| key < f.as_str()) {
                return Err(
                    error(
                        format!(
                            "{key} is before the first stored month {first}, whose cumulated overtime holds the overtime from before; use --force to start over from {key}"
                        )
                    )
                );
            }
        }

        // Without a month before, this month starts over, dropping any overtime from before.
        let start = match entries.range(..key.to_owned()).next_back() {
            Some((k, before)) => parse(k, &before.cumulated)?,
            None => StatusTime::default(),
        };
        let stored = MonthOvertime {
            overtime: overtime.to_string(),
            cumulated: (start + overtime).to_string(),
        };
        entries.insert(key.to_owned(), stored.to_owned());

        // Later months build on this one.
        let mut cumulated = parse(key, &stored.cumulated)?;
        for (k, e) in entries.range_mut((std::ops::Bound::Excluded(key.to_owned()), std::ops::Bound::Unbounded)) {
            cumulated += parse(k, &e.overtime)?;
            e.cumulated = cumulated.to_string();
        }
        Ok(stored)
    }

    /// Stored months whose cumulated overtime doesn't follow from the month before.
    pub fn mismatches(&self) -> Result<Vec<String>, TrackerError> {
        let entries = self.read()?;
        let mut notes = Vec::new();
        let mut before: Option<StatusTime> = None;
        for (k, e) in &entries {
            let cumulated = parse(k, &e.cumulated)?;
            if let Some(b) = before {
                let expected = b + parse(k, &e.overtime)?;
                if expected != cumulated {
                    notes.push(
                        format!("overtime of {k}: cumulated {cumulated} doesn't match {expected} from the month before")
                    );
                }
            }
            before = Some(cumulated);
        }
        Ok(notes)
    }

    /// Overtime accumulated before `start`: the cumulated overtime of the last stored month
    /// before it, plus the overtime of the days between that month and `start`. `None` without
    /// a stored month before `start`.
    pub fn carry(
        &self,
        folder: Folder,
        settings: &Settings,
        start: NaiveDate
    ) -> Result<Option<StatusTime>, TrackerError> {
        let entries = self.read()?;
        let Some((k, e)) = entries.range(..key(start.year(), start.month())).next_back() else {
            return Ok(None);
        };
        let cumulated = parse(k, &e.cumulated)?;

        let from = NaiveDate::parse_from_str(&format!("{k}-01"), "%Y-%m-%d").unwrap() +
        Months::new(1);
        let dates: Vec<_> = from
            .iter_days()
            .take_while(|d| d < &start)
            .collect();
        if dates.is_empty() {
            return Ok(Some(cumulated));
        }
        let data = TimeDataWeekly::builder().folder(folder).week_dates(0, dates).build()?;
        let between = StatusWeekly::builder().data(data).settings(settings.to_owned()).build()?;
        Ok(Some(cumulated + between.overtime))
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ Duration, NaiveDate };

    use super::{ Entries, MonthOvertime, Overtime };
    use crate::{ Settings, StatusTime, TrackerError };

    fn time(s: &str) -> StatusTime {
        s.parse().unwrap()
    }

    fn entry(overtime: &str, cumulated: &str) -> MonthOvertime {
        MonthOvertime { overtime: overtime.to_owned(), cumulated: cumulated.to_owned() }
    }

    fn months(entries: &Entries) -> Vec<(&str, &str, &str)> {
        entries
            .iter()
            .map(|(k, e)| (k.as_str(), e.overtime.as_str(), e.cumulated.as_str()))
            .collect()
    }

    #[test]
    fn should_start_cumulated_with_first_month() {
        let mut entries = Entries::new();
        Overtime::insert(&mut entries, "2025-01", time("-02:15"), false).unwrap();
        Overtime::insert(&mut entries, "2025-02", time("04:00"), false).unwrap();

        assert_eq!(
            vec![("2025-01", "-02:15", "-02:15"), ("2025-02", "04:00", "01:45")],
            months(&entries)
        );
    }

    #[test]
    fn should_build_on_month_before_and_fill_gap() {
        // The first month carries 12 hours from before.
        let mut entries = Entries::from([
            ("2025-01".to_owned(), entry("-01:30", "10:30")),
            ("2025-03".to_owned(), entry("02:00", "12:30")),
        ]);
        Overtime::insert(&mut entries, "2025-02", time("01:00"), false).unwrap();

        assert_eq!(
            vec![
                ("2025-01", "-01:30", "10:30"),
                ("2025-02", "01:00", "11:30"),
                ("2025-03", "02:00", "13:30")
            ],
            months(&entries)
        );
    }

    #[test]
    fn should_replace_only_with_force_and_update_later_months() {
        let mut entries = Entries::from([
            ("2025-01".to_owned(), entry("-01:30", "10:30")),
            ("2025-02".to_owned(), entry("01:00", "11:30")),
        ]);
        let before = entries.clone();
        assert!(Overtime::insert(&mut entries, "2025-01", time("00:30"), false).is_err());
        assert_eq!(before, entries);

        // Replacing the first month starts over, dropping the 12 hours from before.
        Overtime::insert(&mut entries, "2025-01", time("00:30"), true).unwrap();
        assert_eq!(
            vec![("2025-01", "00:30", "00:30"), ("2025-02", "01:00", "01:30")],
            months(&entries)
        );
    }

    #[test]
    fn should_start_over_before_first_only_with_force() {
        let mut entries = Entries::from([
            ("2025-03".to_owned(), entry("01:00", "20:00")),
            ("2025-04".to_owned(), entry("-02:00", "18:00")),
        ]);
        let before = entries.clone();
        assert!(Overtime::insert(&mut entries, "2025-02", time("03:00"), false).is_err());
        assert_eq!(before, entries);

        Overtime::insert(&mut entries, "2025-02", time("03:00"), true).unwrap();
        assert_eq!(
            vec![
                ("2025-02", "03:00", "03:00"),
                ("2025-03", "01:00", "04:00"),
                ("2025-04", "-02:00", "02:00")
            ],
            months(&entries)
        );
    }

    #[test]
    fn should_store_and_report_mismatches() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        let o = Overtime::new(temp_dir.path());
        o.store(2025, 1, time("01:00"), false)?;
        o.store(2025, 2, time("02:00"), false)?;
        assert!(o.mismatches()?.is_empty());

        let file = temp_dir.path().join("overtime.json");
        let content = std::fs::read_to_string(&file)?;
        assert!(content.contains("\"2025-02\": {\n    \"overtime\": \"02:00\",\n    \"cumulated\": \"03:00\""), "{content}");

        std::fs::write(&file, content.replace("\"03:00\"", "\"04:00\""))?;
        assert_eq!(1, o.mismatches()?.len());
        Ok(())
    }

    #[test]
    fn should_reject_invalid_file() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        let file = temp_dir.path().join("overtime.json");
        let o = Overtime::new(temp_dir.path());

        std::fs::write(&file, r#"{"2025-13": {"overtime": "01:00", "cumulated": "01:00"}}"#)?;
        assert!(o.mismatches().is_err());
        std::fs::write(&file, r#"{"2025-01": {"overtime": "1h", "cumulated": "01:00"}}"#)?;
        assert!(o.mismatches().is_err());
        Ok(())
    }

    #[test]
    fn should_carry_stored_and_days_between() -> Result<(), TrackerError> {
        let temp_dir = tempfile::tempdir()?;
        let o = Overtime::new(temp_dir.path());
        let folder = || temp_dir.path().to_path_buf().into();
        let settings = Settings::default();
        let date = |m, d| NaiveDate::from_ymd_opt(2022, m, d).unwrap();

        // Nothing stored before.
        assert_eq!(None, o.carry(folder(), &settings, date(3, 1))?);

        o.store(2022, 1, time("10:00"), false)?;
        // Right after the stored month.
        assert_eq!(Some(time("10:00")), o.carry(folder(), &settings, date(2, 1))?);
        // Mid February: the 1st to 4th (Tuesday to Friday) are missing 8 hours each.
        assert_eq!(
            Some(StatusTime::from(Duration::hours(10 - 32))),
            o.carry(folder(), &settings, date(2, 5))?
        );
        // Not counting the stored month itself or later ones.
        assert_eq!(None, o.carry(folder(), &settings, date(1, 15))?);
        Ok(())
    }
}
