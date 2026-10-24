use std::io;

use serde_json::Value;

use super::timesheet::CURRENT_VERSION;
use crate::{ Entry, Timesheet };

use super::UpgradeError;

pub struct Upgrade {}

impl Upgrade {
    /// Read entries of any known version and upgrade them to the current version.
    pub fn upgrade<R: io::Read>(reader: R) -> Result<Timesheet, UpgradeError> {
        let value: Value = serde_json::from_reader(reader)?;
        let mut timesheet = match value {
            Value::Array(_) => Self::to_v1(value)?,
            _ => serde_json::from_value(value)?,
        };
        if timesheet.version == 1 {
            timesheet = Self::to_v2(timesheet);
        }
        if timesheet.version == 2 {
            timesheet = Self::to_v3(timesheet);
        }
        if timesheet.version != CURRENT_VERSION {
            return Err(UpgradeError::UnsupportedVersion(timesheet.version));
        }
        Ok(timesheet)
    }

    /// Versionless: a plain array of entries.
    fn to_v1(value: Value) -> Result<Timesheet, UpgradeError> {
        let data: Vec<Entry> = serde_json::from_value(value)?;
        Ok(Timesheet { entries: data, version: 1, ..Default::default() })
    }

    /// v2 adds the optional `note` to an entry, which v1 entries simply lack.
    fn to_v2(timesheet: Timesheet) -> Timesheet {
        Timesheet { version: 2, ..timesheet }
    }

    /// v3 drops the `id` of an entry. Entries ignore the `id` they are read with, so it is gone
    /// once the timesheet is written again.
    fn to_v3(timesheet: Timesheet) -> Timesheet {
        Timesheet { version: 3, ..timesheet }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::{ Upgrade, UpgradeError };

    const VERSIONLESS: &str =
        "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
    const V2: &str =
        "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53Z\"},{\"id\":2,\"status\":\"Break\",\"time\":\"2022-02-02T04:00:53Z\",\"note\":\"lunch\"}],\"version\":2}";
    const V1: &str =
        "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";

    #[test]
    fn should_upgrade_versionless_to_v1() {
        let value = serde_json::from_str(VERSIONLESS).unwrap();
        let ent = Upgrade::to_v1(value).unwrap();
        assert_eq!(ent.entries.len(), 2);
        assert_eq!(ent.version, 1)
    }

    #[test]
    fn should_upgrade_v1_to_v2() {
        let v1 = serde_json::from_str(V1).unwrap();
        let ent = Upgrade::to_v2(v1);
        assert_eq!(ent.version, 2);
        assert_eq!(ent.entries.len(), 2);
        assert!(ent.entries.iter().all(|e| e.note.is_none()));
    }

    #[test]
    fn should_upgrade_v2_to_v3() {
        let v2 = serde_json::from_str(V2).unwrap();
        let ent = Upgrade::to_v3(v2);
        assert_eq!(ent.version, 3);
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.data[1].note.as_deref(), Some("lunch"));
    }

    #[test]
    fn should_upgrade_versionless_to_current() {
        let ent = Upgrade::upgrade(VERSIONLESS.as_bytes()).unwrap();
        assert_eq!(ent.version, 3);
        assert_eq!(ent.data.len(), 2);
        assert!(ent.data.iter().all(|e| e.note.is_none()));
    }

    #[test]
    fn should_upgrade_v1_to_current() {
        let ent = Upgrade::upgrade(V1.as_bytes()).unwrap();
        assert_eq!(ent.version, 3);
        assert_eq!(ent.data.len(), 2);
    }

    #[test]
    fn should_upgrade_v2_to_current() {
        let ent = Upgrade::upgrade(V2.as_bytes()).unwrap();
        assert_eq!(ent.version, 3);
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.data[1].note.as_deref(), Some("lunch"));
    }

    #[test]
    fn should_drop_ids_of_upgraded_versions() {
        for old in [VERSIONLESS, V1, V2] {
            let ent = Upgrade::upgrade(old.as_bytes()).unwrap();
            let written = serde_json::to_string(&ent).unwrap();
            assert!(!written.contains("\"id\""), "{written}");
            assert!(written.ends_with("\"version\":3}"), "{written}");
        }
    }

    #[test]
    fn should_keep_current_version() {
        let content = json!({
            "data": [{"status": "Break", "time": "2022-02-02T00:00:53Z", "note": "lunch"}],
            "version": 3
        })
        .to_string();
        let ent = Upgrade::upgrade(content.as_bytes()).unwrap();
        assert_eq!(ent.version, 3);
        assert_eq!(ent.data[0].note.as_deref(), Some("lunch"));
    }

    #[test]
    fn should_reject_version_zero() {
        let res = Upgrade::upgrade("{\"data\":[],\"version\":0}".as_bytes());
        assert!(matches!(res, Err(UpgradeError::UnsupportedVersion(0))));
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
