use std::convert::TryInto;

use crate::{
    Settings, TimeDataWeekly, TimeStatus, TimeStatusDaily, TimeStatusError, TimeStatusWeekly,
};

type Result = std::result::Result<TimeStatusWeekly, TimeStatusError>;

#[derive(Default)]
pub struct WeeklyBuilder<'a> {
    data: Option<&'a TimeDataWeekly>,
    settings: Option<&'a Settings>,
}

impl<'a> WeeklyBuilder<'a> {
    pub fn data(&mut self, data: &'a TimeDataWeekly) -> &mut Self {
        log::trace!("set data to: {:?}", data);
        self.data = Some(data);
        self
    }

    pub fn settings(&mut self, settings: &'a Settings) -> &mut Self {
        log::trace!("set settings to: {:?}", settings);
        self.settings = Some(settings);
        self
    }

    pub fn build(&self) -> Result {
        if self.settings.is_none() {
            Err(TimeStatusError::BuilderDataMissing {
                r#type: "settings".to_string(),
            })
        } else if self.data.is_none() {
            Err(TimeStatusError::BuilderDataMissing {
                r#type: "data".to_string(),
            })
        } else if self.data.unwrap().is_empty() {
            Err(TimeStatusError::TimeDataEmpty)
        } else {
            let mut total = TimeStatus::default();
            let mut c_overtime = TimeStatus::default();
            let settings = self.settings.unwrap().to_owned();
            let data = self.data.unwrap().to_owned();
            let ex_total = settings.workperday.monday
                + settings.workperday.tuesday
                + settings.workperday.wednesday
                + settings.workperday.thursday
                + settings.workperday.friday
                + settings.workperday.saturday
                + settings.workperday.sunday;

            data.entries.iter().for_each(|daily_data| {
                let daily_status = TimeStatusDaily::builder()
                    .data(daily_data)
                    .settings(&settings)
                    .build()
                    .unwrap();
                log::trace!("{:?}", daily_status);
                total += daily_status.worktime;
                c_overtime += daily_status.overtime;
            });

            let overtime = total.to_owned() - TimeStatus::minutes(ex_total.into());
            if c_overtime > overtime {
                log::warn!("some days have no tracking data");
            } else if overtime > c_overtime {
                log::warn!(
                    "theres something odd with overtime calculation. Should be {} but got {}",
                    overtime,
                    c_overtime
                );
            }

            // data.entries.iter().for_each(|d: &TimeStatusDaily| {
            //     log::trace!("processing: {:?}", d);
            //     let mut b = TimeDataDaily::builder();
            //     if !d.entries.is_empty() {
            //         let s = b
            //             .data(d.to_owned())
            //             .settings(settings.clone())
            //             .build()
            //             .unwrap();
            //         log::info!(
            //             "got {} working time and {} overtime",
            //             s.worktime,
            //             s.overtime
            //         );
            //         total += s.worktime;
            //         overtime += s.overtime;
            //     } else {
            //         let expected = self
            //             .settings
            //             .as_ref()
            //             .unwrap()
            //             .workperday
            //             .from_date(d.date.unwrap());
            //         if expected >= &0 {
            //             let exh = expected.to_owned() as i64;
            //             overtime -= TimeStatus::from(Duration::minutes(exh));
            //         }
            //     }
            // });

            log::info!("totally {} working time and {} overtime", total, overtime);

            let decimal: f64 = self.datetime_to_decimal(&total);
            let week = data.week.to_owned();
            Ok(TimeStatusWeekly {
                week: week.try_into()?,
                total,
                overtime,
                decimal,
            })
        }
    }

    fn datetime_to_decimal(&self, total: &TimeStatus) -> f64 {
        let hours = total.hours as f64; //.try_into().unwrap();
        let minutes = total.minutes as f64;
        let md = minutes * (1.0 / 60.0);
        hours + md
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        settings::{BreakLimit, WorkPerDayInMinutes},
        test_utils::{self, init},
        Entry, Status, TimeDataDaily,
    };
    use chrono::{Local, TimeZone};
    use std::ops::Add;

    fn get_settings() -> Settings {
        Settings {
            limits: [
                BreakLimit {
                    start: (6 * 60) + 1,
                    minutes: 45,
                },
                BreakLimit {
                    start: (6 * 60),
                    minutes: 15,
                },
            ]
            .to_vec(),
            workperday: WorkPerDayInMinutes {
                friday: 6 * 60,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    fn get_entries(
        day: u8,
        r#break: Option<u8>,
        break_minutes: u8,
        end: u8,
        end_minutes: u8,
    ) -> Vec<Entry> {
        let break_hours = r#break.unwrap_or(4);
        [
            Entry {
                id: 1,
                status: Status::Connect,
                time: Local.ymd(2022, 3, day.into()).and_hms(0, 0, 0),
            },
            Entry {
                id: 2,
                status: Status::Break,
                time: Local.ymd(2022, 3, day.into()).and_hms(4, 0, 0),
            },
            Entry {
                id: 3,
                status: Status::Connect,
                time: Local.ymd(2022, 3, day.into()).and_hms(
                    break_hours.into(),
                    break_minutes.into(),
                    0,
                ),
            },
            Entry {
                id: 4,
                status: Status::End,
                time: Local
                    .ymd(2022, 3, day.into())
                    .and_hms(end.into(), end_minutes.into(), 0),
            },
        ]
        .to_vec()
    }

    #[test]
    fn should_calculate_overtime() -> test_utils::TestResult {
        init();
        let ex_week: i8 = 10;
        let ex_total = TimeStatus::hours(38).add(TimeStatus::minutes(165));
        let ex_overtime = TimeStatus::minutes(165);
        let ex_decimal = 40.75;

        let vec_daily = [
            TimeDataDaily::builder()
                .entries(&get_entries(7, None, 45, 9, 15)) // 30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(8, None, 45, 9, 30)) // 45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(9, None, 45, 8, 15)) // -30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(10, None, 45, 10, 45)) // 120 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(11, None, 0, 6, 0)) // 0 min
                .build()?,
        ]
        .to_vec();
        let data = TimeDataWeekly::builder()
            .year(&2022)
            .week(&ex_week.try_into()?)
            .entries(&vec_daily)
            .build()?;
        let settings = get_settings();
        let status = TimeStatusWeekly::builder()
            .settings(&settings)
            .data(&data)
            .build()?;

        log::debug!("{}", status);

        assert_eq!(ex_week, status.week, "week is not correct");
        assert_eq!(ex_total, status.total, "total time is not correct");
        assert_eq!(ex_overtime, status.overtime, "overtime is not correct");
        assert_eq!(ex_decimal, status.decimal, "decimal is not correct");
        Ok(())
    }

    #[test]
    fn should_calculate_on_point() -> test_utils::TestResult {
        init();
        let ex_week: i8 = 10;
        let ex_total = TimeStatus::hours(38);
        let ex_overtime = TimeStatus::minutes(0);
        let ex_decimal = 38.0;

        let vec_daily = [
            TimeDataDaily::builder()
                .entries(&get_entries(7, None, 45, 9, 15)) // 30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(8, None, 45, 9, 30)) // 45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(9, None, 45, 8, 15)) // -30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(10, None, 45, 8, 0)) // -45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(11, None, 0, 6, 0)) // 0 min
                .build()?,
        ]
        .to_vec();
        let data = TimeDataWeekly::builder()
            .year(&2022)
            .week(&ex_week.try_into()?)
            .entries(&vec_daily)
            .build()?;
        let settings = get_settings();
        let status = TimeStatusWeekly::builder()
            .settings(&settings)
            .data(&data)
            .build()?;

        log::debug!("{}", status);

        assert_eq!(ex_week, status.week, "week is not correct");
        assert_eq!(ex_total, status.total, "total time is not correct");
        assert_eq!(ex_overtime, status.overtime, "overtime is not correct");
        assert_eq!(ex_decimal, status.decimal, "decimal is not correct");
        Ok(())
    }

    #[test]
    fn should_calculate_less() -> test_utils::TestResult {
        init();
        let ex_week: i8 = 10;
        let ex_total = TimeStatus::hours(35);
        let ex_overtime = TimeStatus::minutes(-180);
        let ex_decimal = 35.0;

        let vec_daily = [
            TimeDataDaily::builder()
                .entries(&get_entries(7, None, 45, 9, 30)) // 45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(8, None, 45, 9, 30)) // 45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(9, None, 45, 5, 45)) // -120 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(10, None, 45, 7, 15)) // -90 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(11, None, 0, 6, 0)) // 0 min
                .build()?,
        ]
        .to_vec();
        let data = TimeDataWeekly::builder()
            .year(&2022)
            .week(&ex_week.try_into()?)
            .entries(&vec_daily)
            .build()?;
        let settings = get_settings();
        let status = TimeStatusWeekly::builder()
            .settings(&settings)
            .data(&data)
            .build()?;

        log::debug!("{}", status);

        assert_eq!(ex_week, status.week, "week is not correct");
        assert_eq!(ex_total, status.total, "total time is not correct");
        assert_eq!(ex_overtime, status.overtime, "overtime is not correct");
        assert_eq!(ex_decimal, status.decimal, "decimal is not correct");
        Ok(())
    }

    #[test]
    fn should_calculate_with_missing_day() -> test_utils::TestResult {
        init();
        let ex_week: i8 = 10;
        let ex_total = TimeStatus::hours(30).add(TimeStatus::minutes(15));
        let ex_overtime = TimeStatus::minutes(-(7 * 60) - 45);
        let ex_decimal = 30.25;

        let vec_daily = [
            TimeDataDaily::builder()
                .entries(&get_entries(7, None, 45, 9, 30)) // 45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(8, None, 45, 9, 15)) // 30 min
                .build()?,
            // Day 9 Missing
            TimeDataDaily::builder()
                .entries(&get_entries(10, None, 45, 7, 45)) // -60 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(11, None, 0, 6, 0)) // 0 min
                .build()?,
        ]
        .to_vec();
        let data = TimeDataWeekly::builder()
            .year(&2022)
            .week(&ex_week.try_into()?)
            .entries(&vec_daily)
            .build()?;
        let settings = get_settings();
        let status = TimeStatusWeekly::builder()
            .settings(&settings)
            .data(&data)
            .build()?;

        log::debug!("{}", status);

        assert_eq!(ex_week, status.week, "week is not correct");
        assert_eq!(ex_total, status.total, "total time is not correct");
        assert_eq!(ex_overtime, status.overtime, "overtime is not correct");
        assert_eq!(ex_decimal, status.decimal, "decimal is not correct");
        Ok(())
    }

    #[test]
    fn should_calculate_with_additional_day() -> test_utils::TestResult {
        init();
        let ex_week: i8 = 10;
        let ex_total = TimeStatus::hours(43);
        let ex_overtime = TimeStatus::minutes(300);
        let ex_decimal = 43.0;

        let vec_daily = [
            TimeDataDaily::builder()
                .entries(&get_entries(7, None, 45, 9, 15)) // 30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(8, None, 45, 9, 30)) // 45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(9, None, 45, 8, 15)) // -30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(10, None, 45, 8, 0)) // -45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(11, None, 0, 6, 0)) // 0 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(12, None, 0, 5, 0)) // 300 min
                .build()?,
        ]
        .to_vec();
        let data = TimeDataWeekly::builder()
            .year(&2022)
            .week(&ex_week.try_into()?)
            .entries(&vec_daily)
            .build()?;
        let settings = get_settings();
        let status = TimeStatusWeekly::builder()
            .settings(&settings)
            .data(&data)
            .build()?;

        log::debug!("{}", status);

        assert_eq!(ex_week, status.week, "week is not correct");
        assert_eq!(ex_total, status.total, "total time is not correct");
        assert_eq!(ex_overtime, status.overtime, "overtime is not correct");
        assert_eq!(ex_decimal, status.decimal, "decimal is not correct");
        Ok(())
    }

    #[test]
    fn should_calculate_no_break() -> test_utils::TestResult {
        init();
        let ex_week: i8 = 10;
        let ex_total = TimeStatus::hours(38);
        let ex_overtime = TimeStatus::minutes(0);
        let ex_decimal = 38.0;

        let vec_daily = [
            TimeDataDaily::builder()
                .entries(&get_entries(7, None, 0, 9, 15)) // 30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(8, None, 0, 9, 30)) // 45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(9, None, 0, 8, 15)) // -30 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(10, None, 0, 8, 0)) // -45 min
                .build()?,
            TimeDataDaily::builder()
                .entries(&get_entries(11, None, 0, 6, 0)) // 0 min
                .build()?,
        ]
        .to_vec();
        let data = TimeDataWeekly::builder()
            .year(&2022)
            .week(&ex_week.try_into()?)
            .entries(&vec_daily)
            .build()?;
        let settings = get_settings();
        let status = TimeStatusWeekly::builder()
            .settings(&settings)
            .data(&data)
            .build()?;

        log::debug!("{}", status);

        assert_eq!(ex_week, status.week, "week is not correct");
        assert_eq!(ex_total, status.total, "total time is not correct");
        assert_eq!(ex_overtime, status.overtime, "overtime is not correct");
        assert_eq!(ex_decimal, status.decimal, "decimal is not correct");
        Ok(())
    }
}
