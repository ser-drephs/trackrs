use config::{Map, Value, ValueKind};
use serde::{Deserialize, Serialize};

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
            Value::new(Some(&"start".to_owned()), ValueKind::U64(l.start.into())),
        );
        m.insert(
            "minutes".to_owned(),
            Value::new(
                Some(&"minutes".to_owned()),
                ValueKind::U64(l.minutes.into()),
            ),
        );
        Value::new(Some(&"BreakLimit".to_string()), ValueKind::Table(m))
    }
}
