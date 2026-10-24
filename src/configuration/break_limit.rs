use config::{ Map, Value, ValueKind };
use serde::Serialize;
use serde_derive::Deserialize;

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq, Eq, Default)]
#[allow(unused)]
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
        Value::new(Some(&"BreakLimit".to_string()), ValueKind::Table(m))
    }
}

#[cfg(test)]
mod tests {
    use config::Value;

    use super::BreakLimit;

    #[test]
    fn should_default_to_no_break() {
        assert_eq!(BreakLimit { start: 0, minutes: 0 }, BreakLimit::default());
    }

    #[test]
    fn should_convert_to_config_table() {
        let value: Value = BreakLimit { start: 360, minutes: 30 }.into();
        let table = value.into_table().unwrap();

        assert_eq!(2, table.len());
        assert_eq!(360, table["start"].clone().into_int().unwrap());
        assert_eq!(30, table["minutes"].clone().into_int().unwrap());
    }

    #[test]
    fn should_roundtrip_through_json() {
        let limit = BreakLimit { start: 540, minutes: 45 };
        let json = serde_json::to_string(&limit).unwrap();

        assert_eq!("{\"start\":540,\"minutes\":45}", json);
        assert_eq!(limit, serde_json::from_str::<BreakLimit>(&json).unwrap());
    }
}
