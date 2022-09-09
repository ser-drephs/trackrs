use std::{fs::File, io::Write};

use chrono::{Local, TimeZone};

use trackrs::{TrackerError, TimeDataDaily};

mod common;

type TestResult = std::result::Result<(), TrackerError>;

#[cfg(test)]
mod daily_builder {

    use super::*;

    #[test]
    fn build_without_file() -> TestResult {
        common::setup();
        let temp_dir = tempfile::tempdir()?;
        let daily_r = TimeDataDaily::builder()
            .root(&temp_dir.into_path().into())
            .date(&Local.ymd(2020, 2, 2))
            .build();
        assert!(daily_r.is_ok());
        let daily = daily_r.unwrap();
        assert_eq!(0, daily.len(), "no entries available");
        Ok(())
    }

    #[test]
    fn build_root_not_set_unwind() {
        common::setup();
        let result = common::catch_unwind_silent(|| TimeDataDaily::builder().build().unwrap());
        assert!(result.is_err());
    }

    #[test]
    fn build_with_file() -> TestResult {
        common::setup();
        let temp_dir = tempfile::tempdir()?;
        let time_file = temp_dir.path().join("20220804.json");
        let day = Local.ymd(2022, 8, 4);

        let mut file = File::create(&time_file)?;
        file.write_all("[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}]".as_bytes())?;

        common::wait(1);

        let daily = TimeDataDaily::builder()
            .root(&temp_dir.into_path().into())
            .date(&day)
            .build()?;

        assert_eq!(2, daily.len(), "two entries in file");
        Ok(())
    }
}
