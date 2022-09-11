use crate::{TimeStatusWeekly, Settings, TrackerError, TimeStatus, TimeStatusDaily, TimeDataDaily};

#[derive(Default)]
pub struct WeeklyBuilder {
    settings: Option<Settings>,
    data: Option<TimeStatusWeekly>,
}

impl WeeklyBuilder {
    pub fn data(&mut self, data: TimeStatusWeekly) -> &mut Self {
        self.data = Some(data);
        self
    }

    pub fn settings(&mut self, settings: Settings) -> &mut Self {
        self.settings = Some(settings);
        self
    }

    pub fn build(&self) -> Result<TimeStatusWeekly, TrackerError> {
        let settings = match self.settings.to_owned() {
            Some(s) => s,
            None => {
                return Err(TrackerError::StatusWeeklyError {
                    message: "settings not defined".to_owned(),
                })
            }
        };

        let data = match self.data.to_owned() {
            Some(d) => d,
            None => {
                return Err(TrackerError::StatusWeeklyError {
                    message: "data not defined".to_owned(),
                })
            }
        };

        let mut total = TimeStatus::default();
        let mut overtime = TimeStatus::default();

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

        // let i_32: i32 = total.duration.num_minutes().try_into()?;
        let decimal: f64 = self.datetime_to_decimal(&total);
        let week = data.week.to_owned();
        // let sw = TimeStatusWeekly {
        //     week,
        //     total,
        //     overtime,
        //     decimal: decimal.to_owned(),
        // };

        // Ok(sw)
        Ok(TimeStatusWeekly::default())
    }

    fn datetime_to_decimal(&self, total: &TimeStatus) -> f64 {
        let hours = total.hours as f64; //.try_into().unwrap();
        let minutes = total.minutes as f64;
        let md = minutes * (1.0 / 60.0);
        hours + md
    }
}
