use chrono::{Date, Datelike, IsoWeek, Local, TimeZone, Weekday};

use crate::{Folder, TimeData, TrackerError};

#[derive(Default, Clone)]
pub struct TimeDataWeekly {
    pub entries: Vec<TimeData>,
    pub week: i8,
}

impl TimeDataWeekly {
    pub fn builder() -> TimeDataWeeklyBuilder {
        TimeDataWeeklyBuilder::default()
    }
}

#[derive(Default)]
pub struct TimeDataWeeklyBuilder {
    inner: TimeDataWeekly,
    folder: Option<Folder>,
    week: Option<i8>,
    year: Option<u16>,
    dates: Option<Vec<Date<Local>>>,
}

impl TimeDataWeeklyBuilder {
    pub fn folder(&mut self, folder: Folder) -> &mut Self {
        log::debug!("set time data folder to: {:?}", &folder);
        self.folder = Some(folder);
        self
    }

    pub fn year(&mut self, year: u16) -> &mut Self {
        log::debug!("set year to: {:?}", year);
        self.year = Some(year);
        self
    }

    pub fn week(&mut self, week: &i8, current_week: IsoWeek) -> &mut Self {
        log::debug!("set week to: {:?}", week);
        self.week = match week > &0 {
            true => Some(week.to_owned()),
            false => {
                let cw: i8 = current_week.week().try_into().unwrap();
                Some(cw + week)
            }
        };
        self.inner.week = self.week.unwrap().to_owned();
        self
    }

    pub fn build(&mut self) -> Result<TimeDataWeekly, TrackerError> {
        if self.year.is_none() {
            return Err(TrackerError::TimeDataError {
                message: "time data year not defined".to_owned(),
            });
        };
        if self.week.is_none() {
            return Err(TrackerError::TimeDataError {
                message: "time data week not defined".to_owned(),
            });
        }
        if self.folder.is_none() {
            return Err(TrackerError::TimeDataError {
                message: "time data folder is not defined".to_owned(),
            });
        }

        self.assert_relative_week().set_dates()?.set_files()?;
        Ok(self.inner.clone())
    }

    fn assert_relative_week(&mut self) -> &mut Self {
        let week = self.week.unwrap();
        if week < 0 {
            let new_year = self.year.unwrap() - 1;
            self.week(&week, Local.ymd(new_year.into(), 12, 31).iso_week());
            self.year(new_year);
            self.assert_relative_week();
        }
        self
    }

    fn set_files(&mut self) -> Result<&mut Self, TrackerError> {
        let dates = match self.dates.to_owned() {
            Some(d) => d,
            None => {
                return Err(TrackerError::TimeDataError {
                    message: "time data dates are not defined".to_owned(),
                })
            }
        };

        let mut entries: Vec<TimeData> = Default::default();

        dates.iter().for_each(|d| {
            let mut b = TimeData::builder();
            let f = self.folder.as_ref().unwrap().to_owned();
            let mut t = b.folder(f).date(d.to_owned()).build().unwrap();
            t.read_from_file().unwrap();
            let mut v = [t].to_vec();
            entries.append(v.as_mut());
        });
        self.inner.entries = entries;
        Ok(self)
    }

    fn set_dates(&mut self) -> Result<&mut Self, TrackerError> {
        let current_year = self.year.unwrap().into();

        let week = self.week.unwrap().try_into()?;

        let mut weekday = Weekday::Mon;
        let mut dates: Vec<Date<Local>> = Default::default();

        loop {
            let d = Local.isoywd(current_year, week, weekday);
            log::debug!("add {:?} {:?} to dates", weekday, d);
            let mut v = [d].to_vec();
            dates.append(v.as_mut());
            if weekday.succ() != Weekday::Mon {
                weekday = weekday.succ();
            } else {
                break;
            }
        }
        self.dates = Some(dates.clone());

        Ok(self)
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    fn logger() {
        std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    mod builder {

        use chrono::Datelike;

        use super::*;

        #[test]
        fn should_set() {
            let d = Local.ymd(2022, 2, 2);
            let mut builder = TimeDataWeekly::builder();
            builder.year(2022).week(&2, d.iso_week());

            assert!(builder.week.is_some());
            assert_eq!(2, builder.week.unwrap());
        }

        #[test]
        fn should_set_week_by_sub() {
            let d = Local.ymd(2022, 2, 2);
            let mut builder = TimeDataWeekly::builder();
            builder.year(2022).week(&-2, d.iso_week());

            assert!(builder.week.is_some());
            assert_eq!(3, builder.week.unwrap());
        }

        #[test]
        fn should_set_negative_week_by_sub() {
            let d = Local.ymd(2022, 2, 2);
            let mut builder = TimeDataWeekly::builder();
            builder.year(2022).week(&-10, d.iso_week());

            assert!(builder.week.is_some());
            assert_eq!(-5, builder.week.unwrap());
        }

        #[test]
        fn should_set_current_week() {
            let d = Local::now(); // required for test
            let w: i8 = d.iso_week().week().try_into().unwrap();

            let mut builder = TimeDataWeekly::builder();
            builder
                .year(d.year().try_into().unwrap())
                .week(&0, d.iso_week());

            assert!(builder.week.is_some());
            assert_eq!(w, builder.week.unwrap());
        }

        #[test]
        #[should_panic(expected = "year not defined")]
        fn no_year() {
            let mut builder = TimeDataWeekly::builder();
            builder.set_dates().unwrap();
        }

        #[test]
        #[should_panic(expected = "week not defined")]
        fn no_week() {
            let mut builder = TimeDataWeekly::builder();
            builder.year(2022).set_dates().unwrap();
        }

        #[test]
        fn negative_week() -> Result<(), TrackerError> {
            let d = Local.ymd(2022, 2, 2);
            let mut builder = TimeDataWeekly::builder();
            builder
                .year(2022)
                .week(&-60, d.iso_week())
                .assert_relative_week()
                .set_dates()?;
            let dates = builder.dates.unwrap();
            assert_eq!(7, dates.len());
            assert_eq!(7, dates.first().unwrap().day());
            assert_eq!(13, dates.last().unwrap().day());
            assert_eq!(12, dates.last().unwrap().month());
            assert_eq!(2020, dates.last().unwrap().year());
            Ok(())
        }

        #[test]
        fn should_set_dates() -> Result<(), TrackerError> {
            logger();
            let d = Local.ymd(2022, 2, 2);
            let mut builder = TimeDataWeekly::builder();
            builder.year(2022).week(&-2, d.iso_week()).set_dates()?;
            assert!(builder.dates.is_some());

            let dates = builder.dates.unwrap();
            assert_eq!(7, dates.len());
            assert_eq!(17, dates.first().unwrap().day());
            assert_eq!(23, dates.last().unwrap().day());
            Ok(())
        }

        #[test]
        #[should_panic(expected = "folder is not defined")]
        fn no_folder() {
            let mut builder = TimeDataWeekly::builder();
            builder.set_files().unwrap();
        }

        #[test]
        #[should_panic(expected = "dates are not defined")]
        fn no_dates() {
            let mut builder = TimeDataWeekly::builder();
            builder.folder(Folder::default()).set_files().unwrap();
        }

        #[test]
        fn should_set_files() -> Result<(), TrackerError> {
            logger();
            let d = Local.ymd(2022, 2, 2);
            let mut builder = TimeDataWeekly::builder();
            builder
                .folder(Folder::default())
                .year(2022)
                .week(&-2, d.iso_week())
                .set_dates()?
                .set_files()?;
            assert_eq!(7, builder.inner.entries.len());

            let first = builder.inner.entries.first().unwrap();
            let last = builder.inner.entries.last().unwrap();

            assert_eq!("20220117.json", first.file.to_str().unwrap());
            assert_eq!("20220123.json", last.file.to_str().unwrap());
            Ok(())
        }

        #[test]
        fn should_build() -> Result<(), TrackerError> {
            logger();
            let d = Local.ymd(2022, 2, 2);
            let mut builder = TimeDataWeekly::builder();
            let t = builder
                .folder(Folder::default())
                .year(2022)
                .week(&-2, d.iso_week())
                .build()?;
            assert_eq!(7, t.entries.len());

            let first = t.entries.first().unwrap();
            let last = t.entries.last().unwrap();

            assert_eq!("20220117.json", first.file.to_str().unwrap());
            assert_eq!("20220123.json", last.file.to_str().unwrap());
            Ok(())
        }
    }
}
