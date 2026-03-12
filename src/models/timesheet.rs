use chrono::{ DateTime, Duration, TimeDelta, Utc };
use serde::{ Deserialize, Serialize };

use crate::{Status, Entry, TimeData};
use crate::entries::Entries;

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

    pub fn get_start(&self) -> Entry {
        match self.data.iter().find(|x| x.is_action(Status::Start)) {
            Some(s) => s.to_owned(),
            None => panic!("no start entry found in timesheet"),
        }
    }

    pub fn get_end(&self) -> Entry {
        match
        self.data
            .iter()
            .rev()
            .find(|x| x.is_action(Status::End))
        {
            Some(e) => e.to_owned(),
            None => Entry::new_now(Status::End),
        }
    }

    pub fn has_end(&self) -> bool {
        match
        self.data
            .iter()
            .rev()
            .find(|x| x.is_action(Status::End))
        {
            Some(_) => true,
            None => false,
        }
    }

    pub fn get_break(&self) -> Option<Entry> {
        self.data
            .iter()
            .find(|x| x.is_action(Status::Break))
            .cloned()
    }

    pub fn calculate_online(&self) -> Duration {
        let start = &self.get_start();
        let end = &self.get_end();
        let online = *end - *start;
        log::info!("calculated a total of {:?} online time", online);
        online
    }

    pub fn calculate_break(&self) -> Duration {
        if self.get_break().is_none() {
            log::info!("no break entries found in timesheet");
            return Duration::zero();
        }

        // set currently taken break
        let mut break_duration = TimeDelta::zero();

        // temporary break datetime
        let mut break_start: DateTime<Utc> = Utc::now();
        // has break
        let mut have_break = false;

        for n in 0..self.data.len() {
            let current = self.data[n];

            if !have_break && current.is_action(Status::Break) {
                break_start = current.time;
                log::debug!("break at: {}", break_start.time());
                have_break = true;
            } else if have_break && current.is_action(Status::Start) {
                let break_end = current.time;
                log::debug!("connect at: {}", break_end.time());
                let delta = break_end - break_start;
                break_duration = break_duration + delta;
                have_break = false;
            }
        }
        log::info!("a total of {:?} break duration was found", break_duration);
        break_duration
    }

    pub fn calculate_worktime(&self) -> Duration {
        let online = self.calculate_online();
        let break_duration = self.calculate_break();
        let worktime = online - break_duration;
        log::info!("calculated a total of {:?} work time", worktime);
        worktime
    }

    pub fn calculate_overtime(&self, expected_worktime: Duration) -> Duration {
        let worktime = self.calculate_worktime();
        let overtime = worktime - expected_worktime;
        log::info!("calculated a total of {:?} overtime", overtime);
        overtime
    }

    pub fn calculate_estimated_end(
        &self,
        expected_worktime: Duration,
        expected_break: Duration
    ) -> DateTime<Utc> {
        let start = &self.get_start();
        let worktime = self.calculate_worktime();
        let break_duration = self.calculate_break();

        let estimated_worktime = if worktime < expected_worktime {
            expected_worktime
        } else {
            worktime
        };

        let estimated_break = if break_duration < expected_break {
            expected_break
        } else {
            break_duration
        };

        let remaining = (estimated_worktime + estimated_break) - (worktime - break_duration);
        start.time + remaining
    }
}

impl From<TimeData> for Timesheet {
    fn from(data: TimeData) -> Self {
        Timesheet::from(data.entries)
    }
}

impl From<Entries> for Timesheet{
    fn from(data: Entries) -> Self  {
        Timesheet { data: data.data, version: data.version }
    }
}

#[cfg(test)]
mod tests {

}
