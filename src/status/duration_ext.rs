use std::fmt::Error;

use time::Duration;

pub struct DurationExtensions{}

impl DurationExtensions{
    pub fn format<'a>(duration: &'a Duration) -> Result<String, Error> {
        let hours = duration.whole_hours();
        let minutes = duration.whole_minutes() - (60 * hours);
        Ok(format!("{:0>2}:{:0>2}", hours, minutes))
    }
}

#[cfg(test)]
mod tests{
    use time::Duration;

    use super::DurationExtensions;

    #[test]
    fn should_format_duration_minutes(){
        let duration = Duration::minutes(45);
        let repr = DurationExtensions::format(&duration).unwrap();
        assert_eq!(repr, "00:45")
    }


    #[test]
    fn should_format_duration_single_minutes(){
        let duration = Duration::minutes(1);
        let repr = DurationExtensions::format(&duration).unwrap();
        assert_eq!(repr, "00:01")
    }


    #[test]
    fn should_format_duration_hour(){
        let duration = Duration::minutes(60);
        let repr = DurationExtensions::format(&duration).unwrap();
        assert_eq!(repr, "01:00")
    }

    #[test]
    fn should_format_duration_hour_minutes(){
        let duration = Duration::minutes(75);
        let repr = DurationExtensions::format(&duration).unwrap();
        assert_eq!(repr, "01:15")
    }
}