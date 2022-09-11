#[cfg(test)]
mod display {
    use chrono::{Local, TimeZone};
    use colored::control::ShouldColorize;
    use indoc::indoc;
    use trackrs::{
        settings::BreakLimit, Entry, Settings, Status, TimeData, TrackerError,
        settings::WorkPerDayInMinutes, TimeStatusDaily, TimeDataDaily,
    };

    type TestResult = Result<(), TrackerError>;

    fn logger() {
        // std::env::set_var("RUST_LOG", "debug");
        let _ = env_logger::builder().is_test(true).try_init();
    }

    // #[test]
// fn status_daily_temp_end() {
//     logger();
//     let local = chrono::Local::now().sub(Duration::minutes(35));
//     let est_end =
//         TimeStatus::from(local.add(Duration::hours(8).add(Duration::minutes(30))));
//     let data = TimeDataDaily {
//         entries: [Entry {
//             id: 1,
//             status: Status::Connect,
//             time: local,
//         }]
//         .to_vec(),
//         ..Default::default()
//     };
//     let settings = Settings {
//         limits: [BreakLimit {
//             start: 6 * 60,
//             minutes: 30,
//         }]
//         .to_vec(),
//         workperday: WorkPerDayInMinutes {
//             saturday: 8 * 60,
//             sunday: 8 * 60,
//             ..Default::default()
//         },
//         ..Default::default()
//     };

//     let status = TimeStatusDaily::builder()
//         .data(data)
//         .settings(settings)
//         .build()
//         .unwrap();

//     log::debug!("{}", status);

//     let str = format!("{}", status);
//     let lines = str.split('\n').collect::<Vec<&str>>();

//     if ShouldColorize::from_env().should_colorize() {
//         //assert worktime
//         assert_eq!("Work time:   00:05 (\u{1b}[91m-07:55\u{1b}[0m)", lines[0]);
//         //assert online
//         assert_eq!("Online time: 00:35", lines[1]);
//         //assert break
//         assert_eq!("Break:       00:00 (\u{1b}[91m-00:30\u{1b}[0m)", lines[2]);

//         assert!(
//             format!("{}", status).contains(&format!(
//                 "End:         \u{1b}[93m{} (est.)\u{1b}[0m",
//                 est_end
//             )),
//             "Expected 'estimated end' to be: {}, but found: {}",
//             est_end,
//             status.est_end
//         );
//     } else {
//         //assert worktime
//         assert_eq!("Work time:   00:05 (-07:55)", lines[0]);
//         //assert online
//         assert_eq!("Online time: 00:35", lines[1]);
//         //assert break
//         assert_eq!("Break:       00:00 (-00:30)", lines[2]);

//         assert!(
//             format!("{}", status).contains(&format!("End:         {}", est_end)),
//             "Expected 'estimated end' to be: {}', but found: {}",
//             est_end,
//             status.est_end
//         );
//     }
//     assert!(!format!("{}", status).contains("Break taken"));
// }

    #[test]
    fn status_daily_with_remaing_worktime_and_break() -> TestResult {
        logger();
        // let data = TimeDataDaily {
        //     entries: [
        //         Entry::new(0, Status::Connect, Local.ymd(2022, 2, 2).and_hms(8, 3, 0)),
        //         Entry::new(1, Status::Break, Local.ymd(2022, 2, 2).and_hms(12, 3, 0)),
        //         Entry::new(2, Status::Connect, Local.ymd(2022, 2, 2).and_hms(12, 23, 0)),
        //         Entry::new(3, Status::End, Local.ymd(2022, 2, 2).and_hms(14, 45, 0)),
        //     ]
        //     .to_vec(),
        //     ..Default::default()
        // };
        // let settings = Settings {
        //     limits: [BreakLimit {
        //         start: 6 * 60,
        //         minutes: 30,
        //     }]
        //     .to_vec(),
        //     ..Default::default()
        // };

        // let status = TimeStatusDaily::builder()
        //     .data(data)
        //     .settings(settings)
        //     .build()
        //     .unwrap();

        // log::debug!("{}", status);

        // if ShouldColorize::from_env().should_colorize() {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   06:12 (\u{1b}[91m-01:48\u{1b}[0m)
        //         Online time: 06:42
        //         Break:       00:20 (\u{1b}[91m-00:10\u{1b}[0m)

        //         Break taken: 12:03 - 12:23
        //         Started:     08:03
        //         End:         \u{1b}[92m14:45\u{1b}[0m"
        //         ),
        //         format!("{}", status)
        //     );
        // } else {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   06:12 (-01:48)
        //         Online time: 06:42
        //         Break:       00:20 (-00:10)

        //         Break taken: 12:03 - 12:23
        //         Started:     08:03
        //         End:         14:45"
        //         ),
        //         format!("{}", status)
        //     );
        // }
        Ok(())
    }

    #[test]
    fn status_daily_with_overtime_and_more_break() -> TestResult {
        logger();
        // let data = TimeDataDaily {
        //     entries: [
        //         Entry::new(0, Status::Connect, Local.ymd(2022, 2, 2).and_hms(8, 3, 0)),
        //         Entry::new(1, Status::Break, Local.ymd(2022, 2, 2).and_hms(12, 3, 0)),
        //         Entry::new(2, Status::Connect, Local.ymd(2022, 2, 2).and_hms(12, 43, 0)),
        //         Entry::new(3, Status::End, Local.ymd(2022, 2, 2).and_hms(17, 45, 0)),
        //     ]
        //     .to_vec(),
        //     ..Default::default()
        // };
        // let settings = Settings {
        //     limits: [BreakLimit {
        //         start: 6 * 60,
        //         minutes: 30,
        //     }]
        //     .to_vec(),
        //     ..Default::default()
        // };
        // let status = TimeStatusDaily::builder()
        //     .data(data)
        //     .settings(settings)
        //     .build()
        //     .unwrap();

        // log::debug!("{}", status);

        // if ShouldColorize::from_env().should_colorize() {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   09:02 (\u{1b}[92m+01:02\u{1b}[0m)
        //         Online time: 09:42
        //         Break:       00:40 (\u{1b}[93m+00:10\u{1b}[0m)

        //         Break taken: 12:03 - 12:43
        //         Started:     08:03
        //         End:         \u{1b}[92m17:45\u{1b}[0m"
        //         ),
        //         format!("{}", status)
        //     );
        // } else {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   09:02 (+01:02)
        //         Online time: 09:42
        //         Break:       00:40 (+00:10)

        //         Break taken: 12:03 - 12:43
        //         Started:     08:03
        //         End:         17:45"
        //         ),
        //         format!("{}", status)
        //     );
        // }
        Ok(())
    }

    #[test]
    fn status_daily_on_point() -> TestResult {
        logger();
        // let data = TimeDataDaily {
        //     entries: [
        //         Entry::new(0, Status::Connect, Local.ymd(2022, 2, 2).and_hms(8, 3, 0)),
        //         Entry::new(1, Status::Break, Local.ymd(2022, 2, 2).and_hms(12, 3, 0)),
        //         Entry::new(2, Status::Connect, Local.ymd(2022, 2, 2).and_hms(12, 33, 0)),
        //         Entry::new(3, Status::End, Local.ymd(2022, 2, 2).and_hms(16, 33, 0)),
        //     ]
        //     .to_vec(),
        //     ..Default::default()
        // };
        // let settings = Settings {
        //     limits: [BreakLimit {
        //         start: 6 * 60,
        //         minutes: 30,
        //     }]
        //     .to_vec(),
        //     ..Default::default()
        // };

        // let status = TimeStatusDaily::builder()
        //     .data(data)
        //     .settings(settings)
        //     .build()
        //     .unwrap();

        // log::debug!("{}", status);

        // if ShouldColorize::from_env().should_colorize() {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   08:00 (+00:00)
        //             Online time: 08:30
        //             Break:       00:30 (+00:00)

        //             Break taken: 12:03 - 12:33
        //             Started:     08:03
        //             End:         \u{1b}[92m16:33\u{1b}[0m"
        //         ),
        //         format!("{}", status)
        //     );
        // } else {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   08:00 (+00:00)
        //         Online time: 08:30
        //         Break:       00:30 (+00:00)

        //         Break taken: 12:03 - 12:33
        //         Started:     08:03
        //         End:         16:33"
        //         ),
        //         format!("{}", status)
        //     );
        // }
        Ok(())
    }

    #[test]
    fn status_daily_short_day_without_break() -> TestResult {
        logger();
        // let data = TimeDataDaily {
        //     entries: [
        //         Entry::new(0, Status::Connect, Local.ymd(2022, 2, 2).and_hms(8, 22, 0)),
        //         Entry::new(1, Status::End, Local.ymd(2022, 2, 2).and_hms(12, 16, 0)),
        //     ]
        //     .to_vec(),
        //     ..Default::default()
        // };

        // let settings = Settings {
        //     limits: [BreakLimit {
        //         start: 8 * 60,
        //         minutes: 45,
        //     }]
        //     .to_vec(),
        //     workperday: WorkPerDayInMinutes {
        //         wednesday: 6 * 60,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // };

        // let status = TimeStatusDaily::builder()
        //     .data(data)
        //     .settings(settings)
        //     .build()
        //     .unwrap();

        // log::debug!("{}", status);

        // if ShouldColorize::from_env().should_colorize() {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   03:54 (\u{1b}[91m-02:06\u{1b}[0m)
        //             Online time: 03:54
        //             Break:       00:00 (+00:00)

        //             Started:     08:22
        //             End:         \u{1b}[92m12:16\u{1b}[0m"
        //         ),
        //         format!("{}", status)
        //     );
        // } else {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   03:54 (-02:06)
        //         Online time: 03:54
        //         Break:       00:00 (+00:00)

        //         Started:     08:22
        //         End:         12:16"
        //         ),
        //         format!("{}", status)
        //     );
        // }
        Ok(())
    }

    #[test]
    fn status_daily_should_ignore_takeover() -> TestResult {
        logger();
        // let data = TimeDataDaily {
        //     entries: [
        //         Entry::new(0, Status::Connect, Local.ymd(2022, 2, 2).and_hms(8, 22, 0)),
        //         Entry::new(1, Status::End, Local.ymd(2022, 2, 2).and_hms(12, 16, 0)),
        //         Entry::new(2, Status::Takeover, Local.ymd(2022, 2, 2).and_hms(12, 46, 0)),
        //     ]
        //     .to_vec(),
        //     ..Default::default()
        // };

        // let settings = Settings {
        //     limits: [BreakLimit {
        //         start: 8 * 60,
        //         minutes: 45,
        //     }]
        //     .to_vec(),
        //     workperday: WorkPerDayInMinutes {
        //         wednesday: 6 * 60,
        //         ..Default::default()
        //     },
        //     ..Default::default()
        // };

        // let status = TimeStatusDaily::builder()
        //     .data(data)
        //     .settings(settings)
        //     .build()
        //     .unwrap();

        // log::debug!("{}", status);

        // if ShouldColorize::from_env().should_colorize() {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   03:54 (\u{1b}[91m-02:06\u{1b}[0m)
        //             Online time: 03:54
        //             Break:       00:00 (+00:00)

        //             Started:     08:22
        //             End:         \u{1b}[92m12:16\u{1b}[0m"
        //         ),
        //         format!("{}", status)
        //     );
        // } else {
        //     assert_eq!(
        //         indoc!(
        //             "Work time:   03:54 (-02:06)
        //         Online time: 03:54
        //         Break:       00:00 (+00:00)

        //         Started:     08:22
        //         End:         12:16"
        //         ),
        //         format!("{}", status)
        //     );
        // }
        Ok(())
    }
}


// mod logic {

//     use crate::settings::{WorkPerDayInMinutes, BreakLimit};

//     use super::*;

//     #[test]
//     fn should_calculate_overtime() {
//         logger();
//         let data = TimeData {
//             entries: [
//                 Entry {
//                     id: 1,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 7).and_hms(8, 55, 46),
//                 },
//                 Entry {
//                     id: 2,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 7).and_hms(8, 56, 15),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::Break,
//                     time: Local.ymd(2022, 2, 7).and_hms(12, 25, 57),
//                 },
//                 Entry {
//                     id: 4,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 7).and_hms(12, 26, 46),
//                 },
//                 Entry {
//                     id: 5,
//                     status: Status::Break,
//                     time: Local.ymd(2022, 2, 7).and_hms(12, 28, 7),
//                 },
//                 Entry {
//                     id: 6,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 7).and_hms(12, 58, 7),
//                 },
//                 Entry {
//                     id: 7,
//                     status: Status::Break,
//                     time: Local.ymd(2022, 2, 7).and_hms(17, 0, 7),
//                 },
//                 Entry {
//                     id: 8,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 7).and_hms(17, 15, 7),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::End,
//                     time: Local.ymd(2022, 2, 7).and_hms(18, 27, 40),
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };
//         let settings = Settings {
//             limits: [BreakLimit {
//                 start: 8 * 60,
//                 minutes: 30,
//             }]
//             .to_vec(),
//             workperday: WorkPerDayInMinutes {
//                 monday: 510,
//                 ..Default::default()
//             },
//             ..Default::default()
//         };

//         let status = TimeStatusDaily::builder()
//             .data(data)
//             .settings(settings)
//             .build()
//             .unwrap();

//         log::debug!("{}", status);

//         assert_eq!(
//             Duration::minutes(16).add(Duration::seconds(5)),
//             status.overtime.duration,
//             "expected 0:16 overtime but was {}",
//             status.overtime
//         );
//     }

//     #[test]
//     fn should_calculate_negative_overtime() {
//         logger();
//         let data = TimeData {
//             entries: [
//                 Entry {
//                     id: 1,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 7).and_hms(8, 22, 11),
//                 },
//                 Entry {
//                     id: 2,
//                     status: Status::End,
//                     time: Local.ymd(2022, 2, 7).and_hms(12, 16, 32),
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };
//         let settings = Settings {
//             limits: [
//                 BreakLimit {
//                     start: 0,
//                     minutes: 15,
//                 },
//                 BreakLimit {
//                     start: 8 * 60,
//                     minutes: 45,
//                 },
//             ]
//             .to_vec(),
//             workperday: WorkPerDayInMinutes {
//                 monday: 360,
//                 ..Default::default()
//             },
//             ..Default::default()
//         };

//         let status = TimeStatusDaily::builder()
//             .data(data)
//             .settings(settings)
//             .build()
//             .unwrap();

//         log::debug!("{}", status);

//         assert_eq!(
//             Duration::hours(-2).sub(Duration::minutes(21).sub(Duration::seconds(21))),
//             status.overtime.duration,
//             "expected -2:21 overtime but was {}",
//             status.overtime
//         );
//     }

//     #[test]
//     fn should_calculate_workktime_and_expected_break_with_only_30_minutes_break_taken() {
//         logger();
//         let data = TimeData {
//             entries: [
//                 Entry {
//                     id: 1,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 2).and_hms(8, 0, 0),
//                 },
//                 Entry {
//                     id: 2,
//                     status: Status::Break,
//                     time: Local.ymd(2022, 2, 2).and_hms(12, 0, 0),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 2).and_hms(12, 30, 0),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::End,
//                     time: Local.ymd(2022, 2, 2).and_hms(17, 0, 0),
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };
//         let settings = Settings {
//             limits: [
//                 BreakLimit {
//                     start: 6 * 60,
//                     minutes: 30,
//                 },
//                 BreakLimit {
//                     start: 8 * 60,
//                     minutes: 45,
//                 },
//                 BreakLimit {
//                     start: 10 * 60,
//                     minutes: 60,
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };

//         let status = TimeStatusDaily::builder()
//             .data(data)
//             .settings(settings)
//             .build()
//             .unwrap();

//         log::debug!("{}", status);
//         assert_eq!(
//             Duration::minutes(45),
//             status.exp_break.as_ref().unwrap().duration,
//             "expected 45 minutes break but was {}",
//             status.exp_break.as_ref().unwrap()
//         );
//         assert_eq!(
//             Duration::hours(8).add(Duration::minutes(15)),
//             status.worktime.duration,
//             "expected 8:15 working time but was {}",
//             status.worktime
//         )
//     }

//     #[test]
//     fn should_calculate_workktime_and_expected_break_with_4_hours_break_taken() {
//         logger();
//         let data = TimeData {
//             entries: [
//                 Entry {
//                     id: 1,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 2).and_hms(8, 0, 0),
//                 },
//                 Entry {
//                     id: 2,
//                     status: Status::Break,
//                     time: Local.ymd(2022, 2, 2).and_hms(12, 0, 0),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 2).and_hms(16, 0, 0),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::End,
//                     time: Local.ymd(2022, 2, 2).and_hms(21, 0, 0),
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };
//         let settings = Settings {
//             limits: [
//                 BreakLimit {
//                     start: 6 * 60,
//                     minutes: 30,
//                 },
//                 BreakLimit {
//                     start: 8 * 60,
//                     minutes: 45,
//                 },
//                 BreakLimit {
//                     start: 10 * 60,
//                     minutes: 60,
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };

//         let status = TimeStatusDaily::builder()
//             .data(data)
//             .settings(settings)
//             .build()
//             .unwrap();

//         log::debug!("{}", status);
//         assert_eq!(
//             Duration::minutes(60),
//             status.exp_break.as_ref().unwrap().duration,
//             "expected 1:00 break but was {}",
//             status.exp_break.as_ref().unwrap()
//         );
//         assert_eq!(
//             Duration::hours(9),
//             status.worktime.duration,
//             "expected 9:00 working time but was {}",
//             status.worktime
//         )
//     }

//     #[test]
//     fn should_calculate_workktime_and_expected_break_with_4_hours_break_taken_and_over_10_hours_worktime(
//     ) {
//         logger();
//         let data = TimeData {
//             entries: [
//                 Entry {
//                     id: 1,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 2).and_hms(8, 0, 0),
//                 },
//                 Entry {
//                     id: 2,
//                     status: Status::Break,
//                     time: Local.ymd(2022, 2, 2).and_hms(12, 0, 0),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 2).and_hms(16, 0, 0),
//                 },
//                 Entry {
//                     id: 3,
//                     status: Status::End,
//                     time: Local.ymd(2022, 2, 2).and_hms(23, 0, 0),
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };
//         let settings = Settings {
//             limits: [
//                 BreakLimit {
//                     start: 6 * 60,
//                     minutes: 30,
//                 },
//                 BreakLimit {
//                     start: 8 * 60,
//                     minutes: 45,
//                 },
//                 BreakLimit {
//                     start: 10 * 60,
//                     minutes: 60,
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };

//         let status = TimeStatusDaily::builder()
//             .data(data)
//             .settings(settings)
//             .build()
//             .unwrap();

//         log::debug!("{}", status);
//         assert_eq!(
//             Duration::hours(1),
//             status.exp_break.as_ref().unwrap().duration,
//             "expected 1:00 hour break but was {}",
//             status.exp_break.as_ref().unwrap()
//         );
//         assert_eq!(
//             Duration::hours(11),
//             status.worktime.duration,
//             "expected 11:00 working time but was {}",
//             status.worktime
//         )
//     }

//     #[test]
//     fn should_calculate_workktime_and_no_expected_break_because_less_than_6_hours_worktime() {
//         logger();
//         let data = TimeData {
//             entries: [
//                 Entry {
//                     id: 1,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 4).and_hms(8, 0, 0),
//                 },
//                 Entry {
//                     id: 2,
//                     status: Status::End,
//                     time: Local.ymd(2022, 2, 4).and_hms(14, 0, 0),
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };
//         let settings = Settings {
//             limits: [
//                 BreakLimit {
//                     start: 6 * 60,
//                     minutes: 30,
//                 },
//                 BreakLimit {
//                     start: 8 * 60,
//                     minutes: 45,
//                 },
//                 BreakLimit {
//                     start: 10 * 60,
//                     minutes: 60,
//                 },
//             ]
//             .to_vec(),
//             workperday: WorkPerDayInMinutes {
//                 friday: 6 * 60,
//                 ..Default::default()
//             },
//             threshold_limits: 5,
//             ..Default::default()
//         };

//         let status = TimeStatusDaily::builder()
//             .data(data)
//             .settings(settings)
//             .build()
//             .unwrap();

//         log::debug!("{}", status);
//         assert_eq!(
//             Duration::minutes(0),
//             status.exp_break.as_ref().unwrap().duration,
//             "expected 0:00 hour break but was {}",
//             status.exp_break.as_ref().unwrap()
//         );
//         assert_eq!(
//             Duration::hours(6),
//             status.worktime.duration,
//             "expected 6:00 working time but was {}",
//             status.worktime
//         )
//     }

//     #[test]
//     fn expected_break_not_displayed_correctly_in_status() {
//         logger();
//         let data = TimeData {
//             entries: [
//                 Entry {
//                     id: 1,
//                     status: Status::Connect,
//                     time: Local.ymd(2022, 2, 2).and_hms(8, 0, 0),
//                 },
//                 Entry {
//                     id: 2,
//                     status: Status::End,
//                     time: Local.ymd(2022, 2, 2).and_hms(14, 0, 0),
//                 },
//             ]
//             .to_vec(),
//             ..Default::default()
//         };
//         let settings = Settings {
//             limits: [
//                 BreakLimit {
//                     start: 6 * 60,
//                     minutes: 30,
//                 },
//                 BreakLimit {
//                     start: 8 * 60,
//                     minutes: 45,
//                 },
//                 BreakLimit {
//                     start: 10 * 60,
//                     minutes: 60,
//                 },
//             ]
//             .to_vec(),
//             workperday: WorkPerDayInMinutes {
//                 friday: 6 * 60,
//                 ..Default::default()
//             },
//             threshold_limits: 5,
//             ..Default::default()
//         };

//         let status = TimeStatusDaily::builder()
//             .data(data)
//             .settings(settings)
//             .build()
//             .unwrap();

//         log::debug!("{}", status);
//         assert_eq!(
//             Duration::minutes(45),
//             status.exp_break.as_ref().unwrap().duration,
//             "expected 0:45 hour break but was {}",
//             status.exp_break.as_ref().unwrap()
//         );
//     }
// }
