use time::Duration;

use super::{break_limit::BreakLimit, Configuration};

pub trait BreakLimitExtensions {
    fn limit_by_start(&self, config: &Configuration, start: &Duration) -> Option<Duration>;
}

impl BreakLimitExtensions for Vec<BreakLimit> {
    fn limit_by_start(&self, config: &Configuration, start: &Duration) -> Option<Duration> {
        let mut limits = config.limits.clone();
        limits.sort_by(|l, r| r.start.partial_cmp(&l.start).unwrap());

        match limits.iter().find(|x| start >= &Duration::minutes(x.start.into())) {
            Some(res) => {
                let dur = Duration::minutes(res.minutes.into());
                log::debug!("should take break of '{}'", dur);
                Some(Duration::minutes(res.minutes.into()))
            }
            None => {
                log::debug!("should not take break");
                None
            }
        }
    }
}