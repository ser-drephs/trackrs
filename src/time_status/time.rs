use crate::Entry;
use chrono::{DateTime, Duration, Local, Timelike};
use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct TimeStatus {
    pub duration: Duration,
    pub hours: i64,
    pub minutes: i64,
}

impl TimeStatus {
    pub fn now() -> TimeStatus {
        TimeStatus::from(Local::now())
    }
}

impl std::fmt::Display for TimeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut negative = false;

        let hours = if self.hours < 0 {
            negative = true;
            -self.hours
        } else {
            self.hours
        };

        let minutes = if self.minutes < 0 {
            negative = true;
            -self.minutes
        } else {
            self.minutes
        };

        if negative {
            write!(f, "-{:0>2}:{:0>2}", hours, minutes)
        } else {
            write!(f, "{:0>2}:{:0>2}", hours, minutes)
        }
    }
}

impl From<&Entry> for TimeStatus {
    fn from(e: &Entry) -> Self {
        let d = Duration::seconds(e.time.num_seconds_from_midnight().into());
        TimeStatus {
            duration: d,
            minutes: d.num_minutes() % 60,
            hours: d.num_hours(),
        }
    }
}

impl From<Entry> for TimeStatus {
    fn from(e: Entry) -> Self {
        let d = Duration::seconds(e.time.num_seconds_from_midnight().into());
        TimeStatus {
            duration: d,
            minutes: d.num_minutes() % 60,
            hours: d.num_hours(),
        }
    }
}

impl From<DateTime<Local>> for TimeStatus {
    fn from(time: DateTime<Local>) -> Self {
        let duration = Duration::seconds(time.num_seconds_from_midnight().into());
        TimeStatus {
            duration,
            minutes: duration.num_minutes() % 60,
            hours: duration.num_hours(),
        }
    }
}

impl From<Duration> for TimeStatus {
    fn from(duration: Duration) -> Self {
        TimeStatus {
            duration,
            minutes: duration.num_minutes() % 60,
            hours: duration.num_hours(),
        }
    }
}

impl From<TimeStatus> for Duration {
    fn from(s: TimeStatus) -> Self {
        s.duration
    }
}

impl Default for TimeStatus {
    fn default() -> Self {
        Self {
            duration: Duration::seconds(0),
            hours: Default::default(),
            minutes: Default::default(),
        }
    }
}

impl PartialEq for TimeStatus {
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration
    }
}

impl PartialOrd for TimeStatus {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.duration.partial_cmp(&other.duration)
    }
}

impl Sub for TimeStatus {
    type Output = TimeStatus;

    fn sub(self, rhs: Self) -> Self::Output {
        let d = self.duration - rhs.duration;
        TimeStatus::from(d)
    }
}

impl Add for TimeStatus {
    type Output = TimeStatus;

    fn add(self, rhs: Self) -> Self::Output {
        let d = self.duration + rhs.duration;
        TimeStatus::from(d)
    }
}

impl AddAssign for TimeStatus {
    fn add_assign(&mut self, rhs: Self) {
        let duration = self.duration + rhs.duration;
        *self = TimeStatus::from(duration);
    }
}

impl SubAssign for TimeStatus {
    fn sub_assign(&mut self, rhs: Self) {
        let duration = self.duration - rhs.duration;
        *self = TimeStatus::from(duration);
    }
}

impl Mul<i32> for TimeStatus {
    type Output = TimeStatus;

    fn mul(self, rhs: i32) -> Self::Output {
        let d = self.duration.mul(rhs);
        TimeStatus::from(d)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Status;
    use chrono::TimeZone;
    use std::cmp::Ordering;

    #[test]
    fn format() {
        let status = TimeStatus::from(Duration::hours(8).add(Duration::minutes(23)));
        assert_eq!("08:23", format!("{}", status));
    }

    #[test]
    fn format_negative() {
        let status = TimeStatus::from(Duration::hours(-2).add(Duration::minutes(-23)));
        assert_eq!("-02:23", format!("{}", status));
    }

    #[test]
    fn from_entry() {
        let d = Duration::seconds(
            Local
                .ymd(2022, 2, 2)
                .and_hms(8, 3, 0)
                .num_seconds_from_midnight()
                .into(),
        );
        let entry = Entry {
            id: 2,
            status: Status::Disconnect,
            time: Local.ymd(2022, 2, 2).and_hms(8, 3, 0),
        };
        let status = TimeStatus::from(&entry);
        assert!(
            status.duration.ge(&d),
            "duration {} should be greater or equal to {}",
            status.minutes,
            &d.num_minutes()
        );
        let dm = &d.num_minutes() % 60;
        assert!(
            status.minutes.ge(&dm),
            "minutes {} should be greater or equal to {}",
            status.minutes,
            &d.num_minutes()
        );
    }

    #[test]
    fn from_datetime() {
        let l = DateTime::default();
        let d = Duration::seconds(l.num_seconds_from_midnight().into());
        let status = TimeStatus::from(l);
        assert!(status.duration.ge(&d));
        assert!(status.minutes.ge(&(&d.num_minutes() % 60)));
    }

    #[test]
    fn from_duration() {
        let d = Duration::minutes(10);
        let status = TimeStatus::from(d);
        assert!(status.duration.ge(&d));
        assert!(status.minutes.ge(&d.num_minutes()));
    }

    #[test]
    fn sub() {
        let a = TimeStatus::from(Duration::minutes(20));
        let b = TimeStatus::from(Duration::minutes(5));

        let e = TimeStatus::from(Duration::minutes(15));
        assert_eq!(e, a - b);
    }

    #[test]
    fn add() {
        let a = TimeStatus::from(Duration::minutes(20));
        let b = TimeStatus::from(Duration::minutes(5));

        let e = TimeStatus::from(Duration::minutes(25));
        assert_eq!(e, a + b);
    }

    #[test]
    fn addassign() {
        let mut a = TimeStatus::from(Duration::minutes(20));
        let b = TimeStatus::from(Duration::minutes(5));

        a += b;

        let e = TimeStatus::from(Duration::minutes(25));
        assert_eq!(e, a);
    }

    #[test]
    fn addassign_from_default() {
        let mut a = TimeStatus::default();
        let b = TimeStatus::from(Duration::minutes(5));
        let c = TimeStatus::from(Duration::minutes(1));

        a += b;
        a += c;

        let e = TimeStatus::from(Duration::minutes(6));
        assert_eq!(e, a);
    }

    #[test]
    fn subassign() {
        let mut a = TimeStatus::from(Duration::minutes(20));
        let b = TimeStatus::from(Duration::minutes(5));

        a -= b;

        let e = TimeStatus::from(Duration::minutes(15));
        assert_eq!(e, a);
    }

    #[test]
    fn subassign_from_default() {
        let mut a = TimeStatus::default();
        let b = TimeStatus::from(Duration::minutes(5));
        let c = TimeStatus::from(Duration::minutes(1));

        a -= b;
        a -= c;

        let e = TimeStatus::from(Duration::minutes(-6));
        assert_eq!(e, a);
    }

    #[test]
    fn mul() {
        let a = TimeStatus::from(Duration::minutes(20));

        let t1 = a.to_owned().mul(2);
        let t2 = a.mul(-1);

        let e1 = TimeStatus::from(Duration::minutes(40));
        let e2 = TimeStatus::from(Duration::minutes(-20));
        assert_eq!(e1, t1);
        assert_eq!(e2, t2);
    }

    #[test]
    fn cmp() {
        let a = TimeStatus::from(Duration::minutes(20));

        let b1 = TimeStatus::from(Duration::minutes(40));
        let b2 = TimeStatus::from(Duration::minutes(-20));

        assert_eq!(Ordering::Less, a.partial_cmp(&b1).unwrap());
        assert_eq!(Ordering::Greater, a.partial_cmp(&b2).unwrap());
    }

    #[test]
    fn eq() {
        let a = TimeStatus::from(Duration::minutes(20));
        let b = TimeStatus::from(Duration::minutes(20));
        assert_eq!(Ordering::Equal, a.partial_cmp(&b).unwrap());
    }
}
