use std::fs::File;

use chrono::{Date, Local};

use crate::{Entry, TimeDataError};

use super::{Folder, TimeDataDaily};

type Result = std::result::Result<TimeDataDaily, TimeDataError>;

#[derive(Default)]
pub struct DailyBuilder<'a> {
    root: Option<&'a Folder>,
    date: Option<&'a Date<Local>>,
    data: Option<&'a Vec<Entry>>,
}

impl<'a> DailyBuilder<'a> {
    /// Set the time data folder
    pub fn root(&mut self, folder: &'a Folder) -> &mut Self {
        log::trace!(
            "set time data root folder to: {:?}, exists: {}",
            folder,
            folder.exists()
        );
        self.root = Some(folder);
        self
    }

    /// Todo: set date to today
    // pub fn today(&mut self) -> &mut Self {
    //     self.date(&Local::today())
    // }

    /// set a date for the time data
    pub fn date(&mut self, date: &'a Date<Local>) -> &mut Self {
        log::trace!("set date to: {:?}", date);
        self.date = Some(date);
        self
    }

    /// Set entries
    ///
    /// This is not required as build will try to read them from the provided file instead.
    pub fn entries(&mut self, data: &'a Vec<Entry>) -> &mut Self {
        log::trace!("set data to: {:?}", data);
        self.data = Some(data);
        self
    }

    pub fn build(&mut self) -> Result {
        let data = if self.data.is_some() {
            self.data.unwrap().to_owned()
        } else {
            if self.root.is_none() {
                return Err(TimeDataError::RootNotProvided);
            }

            if self.date.is_none() {
                return Err(TimeDataError::DateNotProvided);
            }

            let folder = self.root.unwrap();
            let date = self.date.unwrap();
            let file = folder.join(format!("{}.json", date.format("%Y%m%d")));

            if !file.exists() {
                log::info!("root folder or file not yet created");
                // todo: invoke takeover
                // todo: create folder fs::create_dir_all::<PathBuf>(folder.into()).unwrap();
                Vec::<Entry>::new()
            } else {
                let f = File::open(file)?;
                let mut e: Vec<Entry> = serde_json::from_reader(f)?;
                e.sort_by(|a, b| a.id.partial_cmp(&b.id).unwrap());
                e
            }
        };
        Ok(TimeDataDaily { entries: data })
    }
}

#[cfg(test)]
mod tests {

    use chrono::{DateTime, TimeZone};

    use crate::{test_utils::init, Status};

    use super::*;

    #[test]
    fn error_root_not_set() {
        init();
        let daily = TimeDataDaily::builder().build();
        assert!(daily.is_err());
    }

    #[test]
    fn error_date_not_set() {
        init();
        let daily = TimeDataDaily::builder().root(&"".into()).build();
        assert!(daily.is_err());
    }

    #[test]
    fn build_with_entries() {
        init();
        let data = vec![Entry::builder()
            .id(1)
            .status(Status::Connect)
            .time(DateTime::default())
            .build()
            .unwrap()];
        let daily_r = TimeDataDaily::builder().entries(&data).build();
        assert!(daily_r.is_ok());
        let daily = daily_r.unwrap();
        assert_eq!(1, daily.entries.len(), "one element added");
    }

    #[test]
    fn build_without_entries() {
        init();
        let daily_r = TimeDataDaily::builder()
            .root(&"test".into())
            .date(&Local.ymd(2020, 2, 2))
            .build();
        assert!(daily_r.is_ok(), "folder or file not found but that is ok");
        let daily = daily_r.unwrap();
        assert_eq!(0, daily.entries.len(), "no entries available");
    }
}
