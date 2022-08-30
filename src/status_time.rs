use std::ops::{Add, AddAssign, Mul, Sub, SubAssign};

use chrono::{DateTime, Duration, Local, Timelike};

use crate::Entry;

#[derive(Debug, Clone)]
#[allow(unused)]
pub struct StatusTime {
    inner: Option<Entry>,
    pub duration: Duration,
    pub hours: i64,
    pub minutes: i64,
}

impl StatusTime {
    pub fn now() -> StatusTime {
        StatusTime::from(Local::now())
    }
}

impl std::fmt::Display for StatusTime {
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

impl From<&Entry> for StatusTime {
    fn from(e: &Entry) -> Self {
        let d = Duration::seconds(e.time.num_seconds_from_midnight().into());
        StatusTime {
            duration: d,
            minutes: d.num_minutes() % 60,
            hours: d.num_hours(),
            inner: Some(e.clone()),
        }
    }
}

impl From<DateTime<Local>> for StatusTime {
    fn from(time: DateTime<Local>) -> Self {
        let duration = Duration::seconds(time.num_seconds_from_midnight().into());
        StatusTime {
            duration,
            minutes: duration.num_minutes() % 60,
            hours: duration.num_hours(),
            inner: None,
        }
    }
}

impl From<Duration> for StatusTime {
    fn from(duration: Duration) -> Self {
        StatusTime {
            duration,
            minutes: duration.num_minutes() % 60,
            hours: duration.num_hours(),
            inner: None,
        }
    }
}

impl From<StatusTime> for Duration {
    fn from(s: StatusTime) -> Self {
        s.duration
    }
}

impl Default for StatusTime {
    fn default() -> Self {
        Self {
            duration: Duration::seconds(0),
            hours: Default::default(),
            minutes: Default::default(),
            inner: None,
        }
    }
}

impl PartialEq for StatusTime {
    fn eq(&self, other: &Self) -> bool {
        self.duration == other.duration
    }
}

impl PartialOrd for StatusTime {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.duration.partial_cmp(&other.duration)
    }
}

impl Sub for StatusTime {
    type Output = StatusTime;

    fn sub(self, rhs: Self) -> Self::Output {
        let d = self.duration - rhs.duration;
        StatusTime::from(d)
    }
}

impl Add for StatusTime {
    type Output = StatusTime;

    fn add(self, rhs: Self) -> Self::Output {
        let d = self.duration + rhs.duration;
        StatusTime::from(d)
    }
}

impl AddAssign for StatusTime {
    fn add_assign(&mut self, rhs: Self) {
        let duration = self.duration + rhs.duration;
        *self = StatusTime::from(duration);
    }
}

impl SubAssign for StatusTime {
    fn sub_assign(&mut self, rhs: Self) {
        let duration = self.duration - rhs.duration;
        *self = StatusTime::from(duration);
    }
}

impl Mul<i32> for StatusTime {
    type Output = StatusTime;

    fn mul(self, rhs: i32) -> Self::Output {
        let d = self.duration.mul(rhs);
        StatusTime::from(d)
    }
}

#[cfg(test)]
mod tests {

    use chrono::TimeZone;

    use crate::Status;

    use super::*;

    mod format {

        use super::*;

        #[test]
        fn should_format_status_time() {
            let data = Entry {
                id: 1,
                status: Status::Connect,
                time: Local.ymd(2022, 2, 2).and_hms(8, 3, 0),
            };
            let status = StatusTime::from(&data);
            assert_eq!("08:03", format!("{}", status));
        }
    }

    mod from {

        use super::*;

        #[test]
        fn should_create_from_entry() {
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
            let status = StatusTime::from(&entry);
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
            assert!(status.inner.is_some());
            assert_eq!(2, status.inner.unwrap().id);
        }

        #[test]
        fn should_create_from_datetime() {
            let l = Local::now();
            let d = Duration::seconds(l.num_seconds_from_midnight().into());
            let status = StatusTime::from(l);
            assert!(status.duration.ge(&d));
            assert!(status.minutes.ge(&(&d.num_minutes() % 60)));
            assert!(status.inner.is_none());
        }

        #[test]
        fn duration() {
            let d = Duration::minutes(10);
            let status = StatusTime::from(d);
            assert!(status.duration.ge(&d));
            assert!(status.minutes.ge(&d.num_minutes()));
            assert!(status.inner.is_none());
        }
    }

    mod ops {

        use std::cmp::Ordering;

        use super::*;

        #[test]
        fn should_sub() {
            let a = StatusTime::from(Duration::minutes(20));
            let b = StatusTime::from(Duration::minutes(5));

            let e = StatusTime::from(Duration::minutes(15));
            assert_eq!(e, a - b);
        }

        #[test]
        fn should_add() {
            let a = StatusTime::from(Duration::minutes(20));
            let b = StatusTime::from(Duration::minutes(5));

            let e = StatusTime::from(Duration::minutes(25));
            assert_eq!(e, a + b);
        }

        #[test]
        fn should_addassign() {
            let mut a = StatusTime::from(Duration::minutes(20));
            let b = StatusTime::from(Duration::minutes(5));

            a += b;

            let e = StatusTime::from(Duration::minutes(25));
            assert_eq!(e, a);
        }

        #[test]
        fn should_addassign_from_default() {
            let mut a = StatusTime::default();
            let b = StatusTime::from(Duration::minutes(5));
            let c = StatusTime::from(Duration::minutes(1));

            a += b;
            a += c;

            let e = StatusTime::from(Duration::minutes(6));
            assert_eq!(e, a);
        }

        #[test]
        fn should_subassign() {
            let mut a = StatusTime::from(Duration::minutes(20));
            let b = StatusTime::from(Duration::minutes(5));

            a -= b;

            let e = StatusTime::from(Duration::minutes(15));
            assert_eq!(e, a);
        }

        #[test]
        fn should_subassign_from_default() {
            let mut a = StatusTime::default();
            let b = StatusTime::from(Duration::minutes(5));
            let c = StatusTime::from(Duration::minutes(1));

            a -= b;
            a -= c;

            let e = StatusTime::from(Duration::minutes(-6));
            assert_eq!(e, a);
        }

        #[test]
        fn should_mul() {
            let a = StatusTime::from(Duration::minutes(20));

            let t1 = a.to_owned().mul(2);
            let t2 = a.mul(-1);

            let e1 = StatusTime::from(Duration::minutes(40));
            let e2 = StatusTime::from(Duration::minutes(-20));
            assert_eq!(e1, t1);
            assert_eq!(e2, t2);
        }

        #[test]
        fn should_cmp() {
            let a = StatusTime::from(Duration::minutes(20));

            let b1 = StatusTime::from(Duration::minutes(40));
            let b2 = StatusTime::from(Duration::minutes(-20));

            assert_eq!(Ordering::Less, a.partial_cmp(&b1).unwrap());
            assert_eq!(Ordering::Greater, a.partial_cmp(&b2).unwrap());
        }

        #[test]
        fn should_eq() {
            let a = StatusTime::from(Duration::minutes(20));
            let b = StatusTime::from(Duration::minutes(20));
            assert_eq!(Ordering::Equal, a.partial_cmp(&b).unwrap());
        }
    }
}
