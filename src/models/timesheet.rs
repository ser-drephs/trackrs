use chrono::{ DateTime, Duration, TimeDelta, Utc };
use serde::{ Deserialize, Serialize };

use crate::{ Action, Entry };

const CURRENT_VERSION: u8 = 1;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Timesheet {
    data: Vec<Entry>,
    version: u8,
}

impl Timesheet {
    pub fn new() -> Self {
        Timesheet { data: vec!(), version: CURRENT_VERSION }
    }

    pub fn append(&mut self, entry: &mut Vec<Entry>) -> &mut Self {
        self.data.append(entry);
        self
    }

    pub fn version(&self) -> u8 {
        self.version
    }

    pub fn data(&self) -> &Vec<Entry> {
        self.data.as_ref()
    }

    pub fn sort(&mut self) -> &mut Self {
        self.data.sort();
        self
    }

    pub fn get_break(&self, expected: Duration) -> Duration {
        // set currently taken break
        let mut break_duration = TimeDelta::zero();

        // temporary break datetime
        let mut break_start: DateTime<Utc> = Utc::now();
        // has break
        let mut have_break = false;

        for n in 0..self.data.len() {
            let current = self.data[n];

            if !have_break && current.is_action(Action::Break) {
                break_start = current.timestamp();
                log::debug!("break at: {}", break_start.time());
                have_break = true;
            } else if have_break && current.is_action(Action::Start) {
                let break_end = current.timestamp();
                log::debug!("connect at: {}", break_end.time());
                let delta = break_end - break_start;
                break_duration = break_duration + delta;
                have_break = false;
            }
        }
        log::info!("a total of {:?} break duration was found", break_duration);
        expected - break_duration
    }
}
