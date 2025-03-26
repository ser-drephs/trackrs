use chrono::Duration;

use super::BreakThreshold;

pub trait BreakThresholdExtensions {
    fn limit_by_start(&self, start: &Duration) -> Option<Duration>;
}

impl BreakThresholdExtensions for Vec<BreakThreshold> {
    fn limit_by_start(&self, start: &Duration) -> Option<Duration> {
        let mut limits = self.clone();
        limits.sort_by(|l, r| r.start.partial_cmp(&l.start).unwrap());

        match limits.iter().find(|x| start >= &Duration::minutes(x.start.into())) {
            Some(res) => {
                log::debug!("should take break of '{}' minutes", res.minutes);
                Some(Duration::minutes(res.minutes.into()))
            }
            None => {
                log::debug!("should not take break");
                None
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Duration;

    use crate::BreakThreshold;

    use super::BreakThresholdExtensions;

    #[test]
    fn should_return_for_threshold() {
        let limits = vec![
            BreakThreshold { start: 360, minutes: 10 },
            BreakThreshold { start: 480, minutes: 20 },
            BreakThreshold { start: 120, minutes: 5 }
        ];
        assert_eq!(None, limits.limit_by_start(&Duration::minutes(20)), "no break yet");
        assert_eq!(Some(Duration::minutes(5)), limits.limit_by_start(&Duration::minutes(120)), "exactly 120 minutes - 5 minute break");
        assert_eq!(Some(Duration::minutes(20)), limits.limit_by_start(&Duration::minutes(500)), "more than 480 minutes - 20 minute break");
    }
}
