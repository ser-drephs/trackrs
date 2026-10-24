use std::io;

use serde_json::Value;

use crate::{ Entries, Entry, entries::CURRENT_VERSION };

use super::UpgradeError;

pub struct Upgrade {}

impl Upgrade {
    /// Read entries of any known version and upgrade them to the current version.
    pub fn upgrade<R: io::Read>(reader: R) -> Result<Entries, UpgradeError> {
        let value: Value = serde_json::from_reader(reader)?;
        let mut entries = match value {
            Value::Array(_) => Self::to_v1(value)?,
            _ => serde_json::from_value(value)?,
        };
        if entries.version == 1 {
            entries = Self::to_v2(entries);
        }
        if entries.version != CURRENT_VERSION {
            return Err(UpgradeError::UnsupportedVersion(entries.version));
        }
        Ok(entries)
    }

    /// Versionless: a plain array of entries.
    fn to_v1(value: Value) -> Result<Entries, UpgradeError> {
        let data: Vec<Entry> = serde_json::from_value(value)?;
        Ok(Entries { data, version: 1 })
    }

    /// v2 adds the optional `note` to an entry, which v1 entries simply lack.
    fn to_v2(entries: Entries) -> Entries {
        Entries { version: 2, ..entries }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ Upgrade, UpgradeError };

    const VERSIONLESS: &str =
        "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
    const V1: &str =
        "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";

    #[test]
    fn should_upgrade_versionless_to_v1() {
        let value = serde_json::from_str(VERSIONLESS).unwrap();
        let ent = Upgrade::to_v1(value).unwrap();
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.version, 1)
    }

    #[test]
    fn should_upgrade_v1_to_v2() {
        let v1 = serde_json::from_str(V1).unwrap();
        let ent = Upgrade::to_v2(v1);
        assert_eq!(ent.version, 2);
        assert_eq!(ent.data.len(), 2);
        assert!(ent.data.iter().all(|e| e.note.is_none()));
    }

    #[test]
    fn should_upgrade_versionless_to_current() {
        let ent = Upgrade::upgrade(VERSIONLESS.as_bytes()).unwrap();
        assert_eq!(ent.version, 2);
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.data[1].id, 2);
        assert!(ent.data.iter().all(|e| e.note.is_none()));
    }

    #[test]
    fn should_upgrade_v1_to_current() {
        let ent = Upgrade::upgrade(V1.as_bytes()).unwrap();
        assert_eq!(ent.version, 2);
        assert_eq!(ent.data.len(), 2);
    }

    #[test]
    fn should_keep_v2() {
        let content = json!({
            "data": [{"id": 1, "status": "Break", "time": "2022-02-02T00:00:53Z", "note": "lunch"}],
            "version": 2
        })
        .to_string();
        let ent = Upgrade::upgrade(content.as_bytes()).unwrap();
        assert_eq!(ent.version, 2);
        assert_eq!(ent.data[0].note.as_deref(), Some("lunch"));
    }

    #[test]
    fn should_reject_unknown_version() {
        let content = "{\"data\":[],\"version\":99}";
        let res = Upgrade::upgrade(content.as_bytes());
        assert!(matches!(res, Err(UpgradeError::UnsupportedVersion(99))));
    }

    #[test]
    fn should_fail_on_malformed() {
        let content =
            "{\"data\":[\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"}],\"version\":1}";
        let res = Upgrade::upgrade(content.as_bytes());
        assert!(matches!(res, Err(UpgradeError::ReadError(_))));
    }
}
