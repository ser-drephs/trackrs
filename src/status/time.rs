use std::ops::{ Add, AddAssign, Mul, Sub, SubAssign };

use chrono::{ DateTime, Duration, FixedOffset, Local, Timelike, Utc };

use crate::Entry;

#[derive(Debug, Clone, Default)]
pub struct StatusTime {
    pub duration: Duration,
}

impl StatusTime {
    /// Whole hours of the duration, not wrapped at a day.
    pub fn hours(&self) -> i64 {
        self.duration.num_hours()
    }

    /// Minutes of the duration within its hour, negative for a negative duration.
    pub fn minutes(&self) -> i64 {
        self.duration.num_minutes() % 60
    }

    pub fn now() -> StatusTime {
        StatusTime::from(Local::now())
    }

    /// Running sum of `times` after `start`, e.g. the overtime accumulated up to and including
    /// each day.
    pub fn cumulated<'a>(
        start: StatusTime,
        times: impl IntoIterator<Item = &'a StatusTime>
    ) -> Vec<StatusTime> {
        let mut sum = start;
        times
            .into_iter()
            .map(|t| {
                sum += t.to_owned();
                sum.to_owned()
            })
            .collect()
    }
}

impl std::fmt::Display for StatusTime {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut negative = false;

        let hours = if self.hours() < 0 {
            negative = true;
            -self.hours()
        } else {
            self.hours()
        };

        let minutes = if self.minutes() < 0 {
            negative = true;
            -self.minutes()
        } else {
            self.minutes()
        };

        if negative {
            write!(f, "-{:0>2}:{:0>2}", hours, minutes)
        } else {
            write!(f, "{:0>2}:{:0>2}", hours, minutes)
        }
    }
}

/// Parses signed `HH:mm` as written by `Display`, e.g. `-02:15` or `157:00`.
impl std::str::FromStr for StatusTime {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let err = || format!("'{s}' is not a time like 07:30 or -02:15");
        let (negative, rest) = match s.trim().strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, s.trim()),
        };
        let (hours, minutes) = rest.split_once(':').ok_or_else(err)?;
        let hours: i64 = hours.parse().map_err(|_| err())?;
        let minutes: i64 = minutes.parse().map_err(|_| err())?;
        if minutes >= 60 || hours < 0 || minutes < 0 {
            return Err(err());
        }
        let duration = Duration::minutes(hours * 60 + minutes);
        Ok(StatusTime::from(if negative { -duration } else { duration }))
    }
}

impl From<&Entry> for StatusTime {
    fn from(e: &Entry) -> Self {
        StatusTime::from(e.time)
    }
}

impl From<DateTime<FixedOffset>> for StatusTime {
    /// The local time of day of the time, in its own offset.
    fn from(time: DateTime<FixedOffset>) -> Self {
        StatusTime { duration: Duration::seconds(time.num_seconds_from_midnight().into()) }
    }
}

impl From<DateTime<Utc>> for StatusTime {
    fn from(time: DateTime<Utc>) -> Self {
        StatusTime { duration: Duration::seconds(time.num_seconds_from_midnight().into()) }
    }
}

impl From<DateTime<Local>> for StatusTime {
    fn from(time: DateTime<Local>) -> Self {
        StatusTime { duration: Duration::seconds(time.num_seconds_from_midnight().into()) }
    }
}

impl From<Duration> for StatusTime {
    fn from(duration: Duration) -> Self {
        StatusTime { duration }
    }
}

impl From<StatusTime> for Duration {
    fn from(s: StatusTime) -> Self {
        s.duration
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

    #[test]
    fn should_cumulate() {
        let times: Vec<StatusTime> = [30, -90, 0, 120]
            .iter()
            .map(|m| StatusTime::from(Duration::minutes(*m)))
            .collect();
        let sums: Vec<String> = StatusTime::cumulated(StatusTime::default(), &times)
            .iter()
            .map(|t| t.to_string())
            .collect();
        assert_eq!(vec!["00:30", "-01:00", "-01:00", "01:00"], sums);

        let start = StatusTime::from(Duration::hours(10));
        let sums: Vec<String> = StatusTime::cumulated(start, &times)
            .iter()
            .map(|t| t.to_string())
            .collect();
        assert_eq!(vec!["10:30", "09:00", "09:00", "11:00"], sums);

        assert!(StatusTime::cumulated(StatusTime::default(), &[]).is_empty());
    }

    #[test]
    fn should_parse_what_it_displays() {
        for s in ["00:00", "07:30", "-02:15", "157:00", "-00:30", "-2069:00"] {
            assert_eq!(s, s.parse::<StatusTime>().unwrap().to_string());
        }
        for s in ["", "7", "07:60", "07:-1", "a:00", "--01:00", "01:00:00"] {
            assert!(s.parse::<StatusTime>().is_err(), "{s}");
        }
    }

    mod format {
        use super::*;

        #[test]
        fn should_format_status_time() {
            let data = Entry {
                status: Status::Connect,
                time: Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 0).unwrap().fixed_offset(),
                note: None,
            };
            let status = StatusTime::from(&data);
            assert_eq!("08:03", format!("{}", status));
        }

        #[test]
        fn should_format_minutes_over_an_hour() {
            assert_eq!("01:08", StatusTime::from(Duration::minutes(68)).to_string());
        }

        #[test]
        fn should_ignore_seconds() {
            let duration = Duration::minutes(1) + Duration::seconds(10);

            assert_eq!("00:01", StatusTime::from(duration).to_string());
        }

        #[test]
        fn should_truncate_seconds_instead_of_rounding() {
            let duration = Duration::minutes(1) + Duration::seconds(58);

            assert_eq!("00:01", StatusTime::from(duration).to_string());
        }

        #[test]
        fn should_format_more_than_a_day() {
            assert_eq!("25:15", StatusTime::from(Duration::minutes(25 * 60 + 15)).to_string());
            assert_eq!("157:00", StatusTime::from(Duration::hours(157)).to_string());
        }

        #[test]
        fn should_format_negative_time() {
            assert_eq!("-01:30", StatusTime::from(Duration::minutes(-90)).to_string());
            assert_eq!("-00:30", StatusTime::from(Duration::minutes(-30)).to_string());
            assert_eq!("-26:05", StatusTime::from(Duration::minutes(-(26 * 60 + 5))).to_string());
        }
    }

    mod from {
        use chrono::Utc;

        use super::*;

        #[test]
        fn should_create_from_entry() {
            // The time of day of an entry is its own local one, whatever its offset is.
            let time = FixedOffset::east_opt(2 * 3600)
                .unwrap()
                .with_ymd_and_hms(2022, 2, 2, 8, 3, 5)
                .unwrap();
            let entry = Entry {
                status: Status::Disconnect,
                time,
                note: Some("lunch".to_owned()),
            };
            let status = StatusTime::from(&entry);

            assert_eq!(Duration::seconds(8 * 3600 + 3 * 60 + 5), status.duration);
            assert_eq!(8, status.hours());
            assert_eq!(3, status.minutes());
        }

        #[test]
        fn should_create_from_datetime_with_offset() {
            let time = FixedOffset::east_opt(-5 * 3600)
                .unwrap()
                .with_ymd_and_hms(2022, 2, 2, 23, 30, 15)
                .unwrap();
            let status = StatusTime::from(time);

            // The time of day in its own offset, not the one in UTC.
            assert_eq!(Duration::seconds(23 * 3600 + 30 * 60 + 15), status.duration);
            assert_eq!(23, status.hours());
            assert_eq!(30, status.minutes());
        }

        #[test]
        fn should_create_from_datetime() {
            let status = StatusTime::from(Utc.with_ymd_and_hms(2022, 2, 2, 8, 3, 5).unwrap());

            assert_eq!(Duration::seconds(8 * 3600 + 3 * 60 + 5), status.duration);
            assert_eq!(8, status.hours());
            assert_eq!(3, status.minutes());
        }

        #[test]
        fn should_create_from_datetime_at_midnight() {
            let status = StatusTime::from(DateTime::<Utc>::default());

            assert_eq!(Duration::zero(), status.duration);
            assert_eq!(0, status.hours());
            assert_eq!(0, status.minutes());
        }

        #[test]
        fn should_create_from_duration() {
            let status = StatusTime::from(Duration::minutes(10));

            assert_eq!(Duration::minutes(10), status.duration);
            assert_eq!(0, status.hours());
            assert_eq!(10, status.minutes());
        }

        #[test]
        fn should_create_from_duration_over_a_day() {
            let status = StatusTime::from(Duration::hours(25) + Duration::minutes(15));

            assert_eq!(25, status.hours());
            assert_eq!(15, status.minutes());
        }

        #[test]
        fn should_create_from_negative_duration() {
            let status = StatusTime::from(Duration::minutes(-90));

            assert_eq!(Duration::minutes(-90), status.duration);
            assert_eq!(-1, status.hours());
            assert_eq!(-30, status.minutes());
        }

        #[test]
        fn should_create_from_local_datetime() {
            let time = chrono::Local.with_ymd_and_hms(2022, 2, 2, 8, 3, 5).unwrap();
            let status = StatusTime::from(time);

            assert_eq!(Duration::seconds(8 * 3600 + 3 * 60 + 5), status.duration);
            assert_eq!(8, status.hours());
            assert_eq!(3, status.minutes());
        }

        #[test]
        fn should_convert_into_duration() {
            let d = Duration::minutes(75);

            assert_eq!(d, Duration::from(StatusTime::from(d)));
            assert_eq!(Duration::minutes(-30), StatusTime::from(Duration::minutes(-30)).into());
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
