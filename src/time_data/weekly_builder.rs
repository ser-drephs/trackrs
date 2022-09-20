use super::{TimeDataDaily, TimeDataWeekly};
use crate::{time_data::Folder, TimeDataError};
use chrono::{IsoWeek, Local, TimeZone, Weekday};
use std::convert::TryInto;

type Result = std::result::Result<TimeDataWeekly, TimeDataError>;

#[derive(Default)]
pub struct WeeklyBuilder<'a> {
    root: Option<&'a Folder>,
    year: Option<&'a u16>,
    week: Option<u8>,
    data: Option<&'a Vec<TimeDataDaily>>,
}

impl<'a> WeeklyBuilder<'a> {
    pub fn root(&mut self, folder: &'a Folder) -> &mut Self {
        log::trace!(
            "set time data root folder to: {:?}, exists: {}",
            folder,
            folder.exists()
        );
        self.root = Some(folder);
        self
    }

    pub fn year(&mut self, year: &'a u16) -> &mut Self {
        log::trace!("set year to: {:?}", year);
        self.year = Some(year);
        self
    }

    pub fn week(&mut self, week: &'a u8) -> &mut Self {
        log::trace!("set week to: {:?}", week);
        self.week = Some(week.to_owned());
        self
    }

    /// Set week relative to current week
    ///
    /// Week will be set relative to current years week.
    /// If year is set to some year in the past, this will nonetheless be relative to current week.
    pub fn rel_week(&mut self, week: &'a i8, cweek: &'a IsoWeek) -> &mut Self {
        let w = match week > &0 {
            true => week.to_owned() as u8,
            false => {
                let cw: i8 = cweek.week().try_into().unwrap();
                let w = cw + week;
                if w > 0 {
                    w.to_owned() as u8
                } else {
                    log::warn!("resulting week {} crosses year which is not supported. Define week and year instead.", w);
                    1
                }
            }
        };
        self.week = Some(w);
        self
    }

    pub fn entries(&mut self, data: &'a Vec<TimeDataDaily>) -> &mut Self {
        log::trace!("set data to: {:?}", data);
        self.data = Some(data);
        self
    }

    pub fn build(&mut self) -> Result {
        if self.year.is_none() {
            return Err(TimeDataError::BuilderDataMissing {
                r#type: "year".to_string(),
            });
        }

        if self.week.is_none() {
            return Err(TimeDataError::BuilderDataMissing {
                r#type: "week".to_string(),
            });
        }

        let data = if self.data.is_some() {
            self.data.unwrap().to_owned()
        } else {
            if self.root.is_none() {
                return Err(TimeDataError::BuilderDataMissing {
                    r#type: "root".to_string(),
                });
            }

            let year = *self.year.unwrap() as i32;
            let week = self.week.unwrap() as u32;

            let mut data = Vec::<TimeDataDaily>::new();
            let mut weekday = Weekday::Mon;

            let root = self.root.as_ref().unwrap().to_owned();

            loop {
                let date = Local.isoywd(year, week, weekday);
                log::trace!("add {} {:?} to data", weekday, date);
                // dates.append([date].to_vec().as_mut());
                let mut builder = TimeDataDaily::builder();
                let t = builder.root(root).date(&date).build()?;
                data.append([t].to_vec().as_mut());

                if weekday.succ() != Weekday::Mon {
                    weekday = weekday.succ();
                } else {
                    break;
                }
            }
            data
        };

        Ok(TimeDataWeekly {
            entries: data,
            year: self.year.unwrap().to_owned(),
            week: self.week.unwrap().to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{test_utils::init, Entry, Status};
    use chrono::{DateTime, Datelike};

    #[test]
    fn rel_week() {
        let cweek = Local.ymd(2022, 2, 2).iso_week();
        let mut b = TimeDataWeekly::builder();
        b.year(&2022).rel_week(&-2, &cweek);
        assert!(b.week.is_some());
        assert_eq!(3, b.week.unwrap());
    }

    #[test]
    fn rel_week_min_1() {
        let cweek = Local.ymd(2022, 2, 2).iso_week();
        let mut b = TimeDataWeekly::builder();
        b.year(&2022).rel_week(&-10, &cweek);
        assert!(b.week.is_some());
        assert_eq!(1, b.week.unwrap());
    }

    #[test]
    fn error_root_not_set() {
        init();
        let weekly = TimeDataWeekly::builder().year(&2022).week(&23).build();
        assert!(weekly.is_err());
        assert!(weekly
            .unwrap_err()
            .to_string()
            .contains("root is not provided"))
    }

    #[test]
    fn error_year_not_set() {
        init();
        let weekly = TimeDataWeekly::builder().week(&23).build();
        assert!(weekly.is_err());
        assert!(weekly
            .unwrap_err()
            .to_string()
            .contains("year is not provided"))
    }

    #[test]
    fn error_week_not_set() {
        init();
        let weekly = TimeDataWeekly::builder().year(&2021).build();
        assert!(weekly.is_err());
        assert!(weekly
            .unwrap_err()
            .to_string()
            .contains("week is not provided"))
    }

    #[test]
    fn build_with_entries() {
        let data = vec![Entry::new(1, Status::Connect, DateTime::default())];
        let daily = TimeDataDaily::builder().entries(&data).build().unwrap();
        let data = vec![daily];
        let weekly_r = TimeDataWeekly::builder()
            .year(&2022)
            .week(&23)
            .entries(&data)
            .build();
        assert!(weekly_r.is_ok());
        let weekly = weekly_r.unwrap();
        assert_eq!(1, weekly.entries.len(), "one element added");
    }

    #[test]
    fn build_without_entries() {
        let weekly_r = TimeDataWeekly::builder()
            .root(&"test".into())
            .year(&2022)
            .week(&23)
            .build();
        assert!(weekly_r.is_ok());
        let weekly = weekly_r.unwrap();
        assert_eq!(7, weekly.entries.len(), "entries per day available");
    }
}
