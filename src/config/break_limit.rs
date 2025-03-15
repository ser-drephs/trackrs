use config::{ Map, Value, ValueKind };
use serde::{ Deserialize, Serialize };
use time::{ Duration, Weekday };

use super::Configuration;

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct BreakLimit {
    pub start: u16,
    pub minutes: u8,
}

impl From<BreakLimit> for config::Value {
    fn from(l: BreakLimit) -> Self {
        let mut m = Map::new();
        m.insert(
            "start".to_owned(),
            Value::new(Some(&"start".to_owned()), ValueKind::U64(l.start.into()))
        );
        m.insert(
            "minutes".to_owned(),
            Value::new(Some(&"minutes".to_owned()), ValueKind::U64(l.minutes.into()))
        );
        Value::new(Some(&"limits".to_owned()), ValueKind::Table(m))
    }
}

pub trait BreakLimitExtensions {
    fn limit_by_start(&self, config: &Configuration, start: &Duration) -> Option<BreakLimit>;
}

impl BreakLimitExtensions for Vec<BreakLimit> {
    fn limit_by_start(&self, config: &Configuration, start: &Duration) -> Option<BreakLimit> {
        let mut limits = config.limits.clone();
        limits.sort_by(|l, r| r.start.partial_cmp(&l.start).unwrap());

        match limits.iter().find(|x| start >= &Duration::minutes(x.start.into())) {
            Some(res) => {
                let dur = Duration::minutes(res.minutes.into());
                log::debug!("should take break of '{}'", dur);
                Some(res.clone())
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
    use config::{ Config, File, FileFormat };
    use serde::Deserialize;

    use crate::config::break_limit::BreakLimit;

    #[derive(Debug, Deserialize)]
    struct Dummy {
        limits: Vec<BreakLimit>,
    }

    #[test]
    fn should_deserialize_single_element() {
        let settings = Config::builder()
            .add_source(File::from_str("{\"start\":250,\"minutes\":20}", FileFormat::Json))
            .build()
            .unwrap();

        let res = settings.try_deserialize::<BreakLimit>();
        assert!(res.is_ok(), "{:?}", res.err());
        let limit = res.unwrap();
        assert_eq!(limit.minutes, 20)
    }

    #[test]
    fn should_deserialize_multiple_elements() {
        let settings = Config::builder()
            .add_source(
                File::from_str(
                    "{\"limits\":[{\"start\":250,\"minutes\":20}, {\"start\":270,\"minutes\":40}]}",
                    FileFormat::Json
                )
            )
            .build()
            .unwrap();

        let res = settings.try_deserialize::<Dummy>();
        assert!(res.is_ok(), "{:?}", res.err());
        let dummy = res.unwrap();
        assert_eq!(dummy.limits.len(), 2);
        assert_eq!(dummy.limits[0].start, 250)
    }

    #[test]
    fn should_accept_default_config() {
        let limits = vec![BreakLimit { minutes: 10, start: 120 }];
        let settings = Config::builder()
            .set_default("limits", limits)
            .unwrap()
            .add_source(File::from_str("{}", FileFormat::Json))
            .build()
            .unwrap();

        let res = settings.try_deserialize::<Dummy>();
        assert!(res.is_ok(), "{:?}", res.err());
        let dummy = res.unwrap();
        assert_eq!(dummy.limits[0].minutes, 10)
    }
}
