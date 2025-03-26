use config::{ Map, Value, ValueKind };
use serde::{ Deserialize, Serialize };

#[derive(Debug, Deserialize, Serialize, Default, Clone)]
pub struct BreakThreshold {
    pub start: u16,
    pub minutes: u8,
}

impl From<BreakThreshold> for config::Value {
    fn from(l: BreakThreshold) -> Self {
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

#[cfg(test)]
mod tests {
    use config::{ Config, File, FileFormat };
    use nameof::name_of;
    use serde::Deserialize;

    use super::BreakThreshold;

    #[derive(Debug, Deserialize)]
    struct Dummy {
        limits: Vec<BreakThreshold>,
    }

    #[test]
    fn should_deserialize_single_element() {
        let settings = Config::builder()
            .add_source(File::from_str("{\"start\":250,\"minutes\":20}", FileFormat::Json))
            .build()
            .unwrap();

        let res = settings.try_deserialize::<BreakThreshold>();
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
        let limits = vec![BreakThreshold { minutes: 10, start: 120 }];
        let settings = Config::builder()
            .set_default(name_of!(limits in Dummy), limits)
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
