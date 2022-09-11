use crate::{Settings, Status, TimeDataDaily, TimeStatus, TimeStatusDaily, TimeStatusError};
use chrono::{DateTime, Duration, Local};
use std::ops::Add;

type StatusResult = std::result::Result<TimeStatus, TimeStatusError>;
type Result = std::result::Result<TimeStatusDaily, TimeStatusError>;

#[derive(Default)]
pub struct TimeStatusDailyBuilder<'a> {
    data: Option<&'a TimeDataDaily>,
    settings: Option<&'a Settings>,
}

impl<'a> TimeStatusDailyBuilder<'a> {
    pub fn settings(&mut self, settings: &'a Settings) -> &mut Self {
        log::trace!("set settings to: {:?}", settings);
        self.settings = Some(settings);
        self
    }

    pub fn data(&mut self, data: &'a TimeDataDaily) -> &mut Self {
        log::trace!("set data to: {:?}", data);
        self.data = Some(data);
        self
    }

    pub fn build(&self) -> Result {
        if self.settings.is_none() {
            return Err(TimeStatusError::BuilderDataMissing {
                r#type: "settings".to_string(),
            });
        }

        if self.data.is_none() {
            Err(TimeStatusError::BuilderDataMissing {
                r#type: "data".to_string(),
            })
        } else if self.data.unwrap().is_empty() {
            Err(TimeStatusError::TimeDataEmpty)
        } else {
            let start = self.get_start()?;
            let (first_break, total_break) = self.get_break()?;
            let (end, has_end) = self.get_end()?;
            let online = self.get_online(&start, &end)?;
            let exworktime = self.get_exworktime()?;
            let exbreak = self.get_exbreak(&online, &exworktime)?;
            let cbreak = self.get_cbreak(&total_break, &exbreak)?;
            let worktime = self.get_worktime(&online, &cbreak)?;
            let overtime = self.get_overtime(&worktime, &exworktime)?;
            let esend = self.get_esend(&start, &total_break, &exbreak, &exworktime)?;
            Ok(TimeStatusDaily {
                start,
                end,
                has_end,
                r#break: total_break,
                fbreak: first_break,
                online,
                exworktime,
                exbreak,
                cbreak,
                worktime,
                overtime,
                esend,
            })
        }
    }

    fn has_connect(&self) -> std::result::Result<bool, TimeStatusError> {
        let has_connect = match self.data.as_ref() {
            Some(d) => d.entries.iter().any(|x| x.status == Status::Connect),
            None => return Err(TimeStatusError::ConnectNotFound),
        };
        log::trace!("check if any connect entry is present? {}", has_connect);
        Ok(has_connect)
    }

    fn get_start(&self) -> StatusResult {
        log::trace!("try get start from data");
        self.has_connect()?;
        let data = self.data.unwrap();
        let start: TimeStatus = data
            .entries
            .iter()
            .find(|x| x.status == Status::Connect)
            .unwrap()
            .into();
        log::info!("connect at: {}", start);
        Ok(start)
    }

    /// get end - returns end time and true if actually end entry was used, not temporary end.
    fn get_end(&self) -> std::result::Result<(TimeStatus, bool), TimeStatusError> {
        log::trace!("try get end from data");
        let data = self.data.unwrap();
        let mut has_end = false;
        let end = match data.entries.iter().find(|x| x.status == Status::End) {
            Some(c) => {
                log::info!("end at: {}", c.time.time());
                log::info!("finished reading time data for {}", c.time.date());
                has_end = true;
                c.into()
            }
            None => {
                log::trace!("no end entry found threrefore create a temporary one");
                TimeStatus::now()
            }
        };
        log::trace!("end at: {} is end? {}", end, has_end);
        Ok((end, has_end))
    }

    /// get breaks - returns first break, total break
    fn get_break(&self) -> std::result::Result<(TimeStatus, TimeStatus), TimeStatusError> {
        log::trace!("try get breaks from data");
        // define total break duration
        let mut bd = Duration::seconds(0);
        // temporary break datetime
        let mut bt: DateTime<Local> = DateTime::default();

        let mut has_break = false;
        let mut first_break: Option<TimeStatus> = None;

        let data = self.data.unwrap();
        for n in 0..data.len() {
            if !has_break {
                // get break entry
                if data.entries[n].status == Status::Break {
                    // temp save time
                    bt = data.entries[n].time;
                    log::info!("break at: {}", bt.time());
                    has_break = true;
                    if first_break.is_none() {
                        first_break = Some(data.entries[n].clone().into());
                    }
                }
            } else if has_break && data.entries[n].status == Status::Connect {
                // get next connect
                let ct = data.entries[n].time;
                log::info!("connect at: {}", ct.time());
                // caluclate break duration
                let cbd = ct - bt;
                // add to general break
                bd = bd + cbd;
                has_break = false;
            }
        }
        let total_break: TimeStatus = bd.into();
        log::info!("a total of {:?} break duration was found", bd);
        log::trace!(
            "first break at: {} total break: {}",
            first_break.as_ref().unwrap(),
            total_break
        );
        Ok((first_break.unwrap(), total_break))
    }

    /// get online time with start time and end time,
    fn get_online(&self, start: &TimeStatus, end: &TimeStatus) -> StatusResult {
        let end = end.clone();
        let online = end - start.clone();
        log::trace!("calculated online time: {}", online);
        Ok(online)
    }

    /// get worktime with online time and calculated break time.
    fn get_worktime(&self, online: &TimeStatus, cbreak: &TimeStatus) -> StatusResult {
        let worktime = online.clone() - cbreak.clone();
        log::trace!("calculated work time: {}", worktime);
        Ok(worktime)
    }

    /// get overtime with worktime and expected worktime
    fn get_overtime(&self, worktime: &TimeStatus, exworktime: &TimeStatus) -> StatusResult {
        let overtime = worktime.clone() - exworktime.clone();
        log::trace!("calculated over time: {}", overtime);
        Ok(overtime)
    }

    /// get calculated break with total break and expected break.
    fn get_cbreak(&self, total_break: &TimeStatus, exbreak: &TimeStatus) -> StatusResult {
        let cbreak = if total_break >= exbreak {
            total_break
        } else {
            exbreak
        };
        log::trace!("calculated break: {}", cbreak);
        Ok(cbreak.clone())
    }

    fn get_exworktime(&self) -> StatusResult {
        log::trace!("try get expected work time from settings");
        let data = self.data.unwrap();
        let settings = self.settings.unwrap();
        let exworktime: TimeStatus = Duration::minutes(
            settings
                .workperday
                .from(data.entries[0].time)
                .to_owned()
                .into(),
        )
        .into();
        log::trace!("selected expected work time: {}", exworktime);
        Ok(exworktime)
    }

    /// get expected break by online time and expected work time. Also values from settings are taken here.
    fn get_exbreak(&self, online: &TimeStatus, exworktime: &TimeStatus) -> StatusResult {
        log::trace!("try get expected break from online time and settings");
        let settings = self.settings.unwrap();

        // get whatever time is heigher, either expected working time for the day or the online time.
        let (duration, threshold) = if online >= exworktime {
            log::trace!("online time is higher");
            (
                online.duration,
                Duration::minutes(settings.threshold_limits.into()),
            )
        } else {
            log::trace!("expected work time is higher");
            (exworktime.duration, Duration::seconds(0))
        };
        log::trace!("duration for selection: {}", duration);

        let mut break_limits = settings.limits.to_owned();
        break_limits.sort_by(|x, y| y.start.partial_cmp(&x.start).unwrap());
        let exbreak: TimeStatus = match break_limits
            .iter_mut()
            .find(|x| duration - threshold >= Duration::minutes(x.start.to_owned().into()))
        {
            Some(break_limit) => {
                log::debug!("should take a break of {}", break_limit.minutes);
                Duration::minutes(break_limit.minutes.into()).into()
            }
            None => {
                log::debug!("should not take a break");
                Duration::minutes(0).into()
            }
        };
        log::trace!("selected expected break: {}", exbreak);
        Ok(exbreak)
    }

    /// get estimated end with start, total break, expected break and expected worktime
    fn get_esend(
        &self,
        start: &TimeStatus,
        total_break: &TimeStatus,
        exbreak: &TimeStatus,
        exworktime: &TimeStatus,
    ) -> StatusResult {
        let total_worktime = if total_break > exbreak {
            log::trace!("total break is higher");
            exworktime.clone().add(total_break.clone())
        } else {
            log::trace!("expected break is higher");
            exworktime.clone().add(exbreak.clone())
        };
        log::trace!("calculated total work time: {}", total_worktime);
        let esend = start.clone().add(total_worktime);
        log::trace!("calculated estimated end: {}", esend);
        Ok(esend)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::{
        settings::{BreakLimit, WorkPerDayInMinutes},
        test_utils::{self, init},
        Entry,
    };
    use chrono::TimeZone;

    fn test_date() -> chrono::Date<chrono::Local> {
        Local.ymd(2022, 2, 2)
    }

    #[test]
    fn has_connect() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![
            Entry::new(1, Status::Connect, test_date().and_hms(8, 3, 0)),
            Entry::new(2, Status::Disconnect, test_date().and_hms(15, 6, 0)),
        ];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);
        assert!(builder.has_connect().is_ok());
        Ok(())
    }

    #[test]
    fn has_no_connect() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![
            Entry::new(1, Status::Break, test_date().and_hms(12, 6, 0)),
            Entry::new(2, Status::Disconnect, test_date().and_hms(15, 6, 0)),
            Entry::new(3, Status::End, test_date().and_hms(16, 6, 0)),
            Entry::new(4, Status::Takeover, test_date().and_hms(17, 6, 0)),
        ];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);
        assert!(builder.has_connect().is_err());
        Ok(())
    }

    #[test]
    fn no_connect() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![Entry::new(
            1,
            Status::Disconnect,
            test_date().and_hms(8, 6, 0),
        )];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let result = TimeStatusDaily::builder()
            .data(&time_data)
            .settings(&settings)
            .build();
        assert!(result.is_err());
        assert!(format!("{}", result.unwrap_err()).contains("no initial connect found in data"));
        Ok(())
    }

    #[test]
    fn get_start() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![
            Entry::new(1, Status::Connect, test_date().and_hms(8, 3, 0)),
            Entry::new(2, Status::Disconnect, test_date().and_hms(15, 6, 0)),
        ];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);
        let exp: TimeStatus = Duration::hours(8).add(Duration::minutes(3)).into();
        let start = builder.get_start()?;

        assert_eq!(exp, start);
        Ok(())
    }

    #[test]
    fn get_end_has_end() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![
            Entry::new(1, Status::Connect, test_date().and_hms(12, 6, 0)),
            Entry::new(2, Status::End, test_date().and_hms(16, 6, 0)),
        ];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);
        let (end, has_end) = builder.get_end()?;
        let exp: TimeStatus = Duration::hours(16).add(Duration::minutes(6)).into();
        assert_eq!(exp, end);
        assert!(has_end);
        Ok(())
    }

    #[test]
    fn get_end_not_has_end() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![
            Entry::new(1, Status::Connect, test_date().and_hms(12, 6, 0)),
            Entry::new(2, Status::Connect, test_date().and_hms(16, 6, 0)),
        ];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);
        let (end, has_end) = builder.get_end()?;
        let exp = TimeStatus::now();
        assert_eq!(exp, end);
        assert!(!has_end);
        Ok(())
    }

    #[test]
    fn get_break() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![
            Entry::new(1, Status::Connect, test_date().and_hms(8, 6, 0)),
            Entry::new(2, Status::Break, test_date().and_hms(11, 6, 0)),
            Entry::new(3, Status::Connect, test_date().and_hms(12, 6, 0)),
        ];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let (first, total) = builder.get_break()?;
        let exp_first: TimeStatus = Duration::hours(11).add(Duration::minutes(6)).into();
        let exp_total: TimeStatus = Duration::hours(1).into();

        assert_eq!(exp_first, first);
        assert_eq!(exp_total, total);
        Ok(())
    }

    #[test]
    fn get_break_multiple_entries() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let data = vec![
            Entry::new(1, Status::Connect, test_date().and_hms(8, 6, 0)),
            Entry::new(2, Status::Break, test_date().and_hms(11, 0, 0)),
            Entry::new(3, Status::Connect, test_date().and_hms(11, 10, 0)),
            Entry::new(4, Status::Break, test_date().and_hms(12, 0, 0)),
            Entry::new(5, Status::Connect, test_date().and_hms(12, 10, 0)),
            Entry::new(6, Status::Break, test_date().and_hms(13, 0, 0)),
            Entry::new(7, Status::Connect, test_date().and_hms(13, 10, 0)),
        ];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let (first, total) = builder.get_break()?;
        let exp_first: TimeStatus = Duration::hours(11).into();
        let exp_total: TimeStatus = Duration::minutes(30).into();

        assert_eq!(exp_first, first);
        assert_eq!(exp_total, total);
        Ok(())
    }

    #[test]
    fn get_online() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let start: TimeStatus = Duration::hours(8).add(Duration::minutes(10)).into();
        let end: TimeStatus = Duration::hours(14).add(Duration::minutes(20)).into();
        let exp: TimeStatus = Duration::hours(6).add(Duration::minutes(10)).into();
        let online = builder.get_online(&start, &end)?;

        assert_eq!(exp, online);
        Ok(())
    }

    #[test]
    fn get_worktime() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let online: TimeStatus = Duration::hours(8).add(Duration::minutes(10)).into();
        let cbreak: TimeStatus = Duration::minutes(20).into();
        let exp: TimeStatus = Duration::hours(7).add(Duration::minutes(50)).into();
        let worktime = builder.get_worktime(&online, &cbreak)?;

        assert_eq!(exp, worktime);
        Ok(())
    }

    #[test]
    fn get_overtime_more() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let worktime: TimeStatus = Duration::hours(9).add(Duration::minutes(10)).into();
        let exworktime: TimeStatus = Duration::hours(8).into();
        let exp: TimeStatus = Duration::hours(1).add(Duration::minutes(10)).into();
        let overtime = builder.get_overtime(&worktime, &exworktime)?;

        assert_eq!(exp, overtime);
        Ok(())
    }

    #[test]
    fn get_overtime_less() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let worktime: TimeStatus = Duration::hours(5).add(Duration::minutes(10)).into();
        let exworktime: TimeStatus = Duration::hours(8).into();
        let exp: TimeStatus = Duration::hours(-2).add(Duration::minutes(-50)).into();
        let overtime = builder.get_overtime(&worktime, &exworktime)?;

        assert_eq!(exp, overtime);
        Ok(())
    }

    #[test]
    fn get_cbreak_no_break() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let total_break: TimeStatus = Duration::minutes(0).into();
        let exbreak: TimeStatus = Duration::minutes(45).into();
        let exp = exbreak.clone();
        let cbreak = builder.get_cbreak(&total_break, &exbreak)?;

        assert_eq!(exp, cbreak);
        Ok(())
    }

    #[test]
    fn get_cbreak_more_exp_break() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let total_break: TimeStatus = Duration::minutes(10).into();
        let exbreak: TimeStatus = Duration::minutes(45).into();
        let exp = exbreak.clone();
        let cbreak = builder.get_cbreak(&total_break, &exbreak)?;

        assert_eq!(exp, cbreak);
        Ok(())
    }

    #[test]
    fn get_cbreak_more_break() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let total_break: TimeStatus = Duration::minutes(90).into();
        let exbreak: TimeStatus = Duration::minutes(45).into();
        let exp = total_break.clone();
        let cbreak = builder.get_cbreak(&total_break, &exbreak)?;

        assert_eq!(exp, cbreak);
        Ok(())
    }

    #[test]
    fn get_exworktime() -> test_utils::TestResult {
        init();
        let settings = Settings {
            workperday: WorkPerDayInMinutes {
                wednesday: 600,
                ..Default::default()
            },
            ..Default::default()
        };
        let data = vec![Entry::new(1, Status::Connect, test_date().and_hms(8, 6, 0))];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let exp: TimeStatus = Duration::minutes(600).into();
        let exworktime = builder.get_exworktime()?;

        assert_eq!(exp, exworktime);
        Ok(())
    }

    #[test]
    fn get_exworktime_different_days() -> test_utils::TestResult {
        init();
        let settings = Settings {
            workperday: WorkPerDayInMinutes {
                wednesday: 480,
                friday: 360,
                ..Default::default()
            },
            ..Default::default()
        };
        let data = vec![Entry::new(1, Status::Connect, test_date().and_hms(8, 6, 0))];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let exp: TimeStatus = Duration::minutes(480).into();
        let exworktime = builder.get_exworktime()?;

        assert_eq!(exp, exworktime);

        let data2 = vec![Entry::new(
            1,
            Status::Connect,
            Local.ymd(2022, 2, 4).and_hms(8, 6, 0),
        )];
        let time_data2 = TimeDataDaily::builder().entries(&data2).build()?;
        let mut builder2 = TimeStatusDaily::builder();
        builder2.data(&time_data2).settings(&settings);

        let exp2: TimeStatus = Duration::minutes(360).into();
        let exworktime2 = builder2.get_exworktime()?;
        assert_eq!(exp2, exworktime2);
        Ok(())
    }

    #[test]
    fn get_exbreak_in_range() -> test_utils::TestResult {
        init();
        let settings = Settings {
            limits: vec![
                BreakLimit {
                    start: 361,
                    minutes: 15,
                },
                BreakLimit {
                    start: 481,
                    minutes: 45,
                },
                BreakLimit {
                    start: 601,
                    minutes: 60,
                },
            ],
            ..Default::default()
        };
        let data = vec![Entry::new(1, Status::Connect, test_date().and_hms(8, 6, 0))];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let online: TimeStatus = Duration::hours(8).add(Duration::minutes(10)).into();
        let exworktime: TimeStatus = Duration::minutes(480).into();
        let exp: TimeStatus = Duration::minutes(45).into();
        let exbreak = builder.get_exbreak(&online, &exworktime)?;

        assert_eq!(exp, exbreak);
        Ok(())
    }

    #[test]
    fn get_exbreak_below_range() -> test_utils::TestResult {
        init();
        let settings = Settings {
            limits: vec![
                BreakLimit {
                    start: 361,
                    minutes: 15,
                },
                BreakLimit {
                    start: 481,
                    minutes: 45,
                },
                BreakLimit {
                    start: 601,
                    minutes: 60,
                },
            ],
            ..Default::default()
        };
        let data = vec![Entry::new(1, Status::Connect, test_date().and_hms(8, 6, 0))];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let online: TimeStatus = Duration::hours(3).add(Duration::minutes(10)).into();
        let exworktime: TimeStatus = Duration::minutes(480).into();
        let exp: TimeStatus = Duration::minutes(15).into();
        let exbreak = builder.get_exbreak(&online, &exworktime)?;

        assert_eq!(exp, exbreak);
        Ok(())
    }

    #[test]
    fn get_exbreak_over_range() -> test_utils::TestResult {
        init();
        let settings = Settings {
            limits: vec![
                BreakLimit {
                    start: 361,
                    minutes: 15,
                },
                BreakLimit {
                    start: 481,
                    minutes: 45,
                },
                BreakLimit {
                    start: 601,
                    minutes: 60,
                },
            ],
            ..Default::default()
        };
        let data = vec![Entry::new(1, Status::Connect, test_date().and_hms(8, 6, 0))];
        let time_data = TimeDataDaily::builder().entries(&data).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let online: TimeStatus = Duration::hours(10).add(Duration::minutes(10)).into();
        let exworktime: TimeStatus = Duration::minutes(480).into();
        let exp: TimeStatus = Duration::minutes(60).into();
        let exbreak = builder.get_exbreak(&online, &exworktime)?;

        assert_eq!(exp, exbreak);
        Ok(())
    }

    #[test]
    fn get_esend() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let start: TimeStatus = Duration::hours(6).add(Duration::minutes(10)).into();
        let total_break: TimeStatus = Duration::minutes(45).into();
        let exbreak: TimeStatus = Duration::minutes(45).into();
        let exworktime: TimeStatus = Duration::minutes(480).into();
        let exp: TimeStatus = Duration::hours(14).add(Duration::minutes(55)).into();
        let esend = builder.get_esend(&start, &total_break, &exbreak, &exworktime)?;

        assert_eq!(exp, esend);
        Ok(())
    }

    #[test]
    fn get_esend_more_exbreak() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let start: TimeStatus = Duration::hours(6).add(Duration::minutes(10)).into();
        let total_break: TimeStatus = Duration::minutes(30).into();
        let exbreak: TimeStatus = Duration::minutes(60).into();
        let exworktime: TimeStatus = Duration::minutes(480).into();
        let exp: TimeStatus = Duration::hours(15).add(Duration::minutes(10)).into();
        let esend = builder.get_esend(&start, &total_break, &exbreak, &exworktime)?;

        assert_eq!(exp, esend);
        Ok(())
    }

    #[test]
    fn get_esend_more_total_break() -> test_utils::TestResult {
        init();
        let settings = Settings::default();
        let time_data = TimeDataDaily::builder().entries(&vec![]).build()?;
        let mut builder = TimeStatusDaily::builder();
        builder.data(&time_data).settings(&settings);

        let start: TimeStatus = Duration::hours(6).add(Duration::minutes(10)).into();
        let total_break: TimeStatus = Duration::minutes(90).into();
        let exbreak: TimeStatus = Duration::minutes(45).into();
        let exworktime: TimeStatus = Duration::minutes(480).into();
        let exp: TimeStatus = Duration::hours(15).add(Duration::minutes(40)).into();
        let esend = builder.get_esend(&start, &total_break, &exbreak, &exworktime)?;

        assert_eq!(exp, esend);
        Ok(())
    }
}
