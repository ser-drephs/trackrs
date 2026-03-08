use std::fs::File;
use std::io;
use std::io::BufReader;
use crate::{entries, Entries, Entry};

use super::UpgradeError;

pub struct Upgrade {}

impl Upgrade {
    pub fn upgrade(raw_string: &str) -> Result<Option<Entries>, UpgradeError> {
        let v1_entries = Self::read_upgrade_to_v1(raw_string)?;
        let v2_entries = match v1_entries {
            Some(entries) => Self::upgrade_to_v2(Some(entries)),
            None => Self::read_upgrade_to_v2(raw_string)
        };
        v2_entries
    }

    /**
     * Tries to read and upgrade the model to version 1 of the format, which adds `version`
     * field and wraps the entries in a `data` field.
     */
    fn read_upgrade_to_v1(raw_string: &str) -> Result<Option<Entries>, UpgradeError> {
        let vec_entries: Result<Vec<Entry>, serde_json::Error> = serde_json::from_str(raw_string);
        if vec_entries.is_err() {
            let err = vec_entries.unwrap_err();
            if err.is_data() || err.is_eof() {
                return Ok(None);
            }
            return Err(UpgradeError::UpgradeV1Error {
                message: err.to_string(),
            });
        }
        Self::upgrade_to_v1(vec_entries.ok())
    }

    /**
     * Tries to upgrade the model to version 1 of the format, which adds `version`
     * field and wraps the entries in a `data` field.
     */
    fn upgrade_to_v1(entries: Option<Vec<Entry>>) -> Result<Option<Entries>, UpgradeError> {
        if entries.is_none() {
            return Ok(None);
        }
        let mut vec_entries = entries.unwrap();
        let mut entries = Entries::new();
        entries.append(&mut vec_entries);
        Ok(Some(entries))
    }

    /**
     * Tries to read and upgrade the model to version 2 of the format, which removes `id` from the entries.
     */
    #[allow(deprecated)]
    fn read_upgrade_to_v2(raw_string: &str) -> Result<Option<Entries>, UpgradeError> {
        let entries: Result<Entries, serde_json::Error> = serde_json::from_str(raw_string);
        if entries.is_err() {
            let err = entries.unwrap_err();
            if err.is_data() || err.is_eof() {
                return Ok(None);
            }
            return Err(UpgradeError::UpgradeV2Error {
                message: err.to_string(),
            });
        }
        Self::upgrade_to_v2(entries.ok())
    }

    #[allow(deprecated)]
    fn upgrade_to_v2(entries: Option<Entries>) -> Result<Option<Entries>, UpgradeError> {
        if entries.is_none() {
            return Ok(None);
        }
        let mut clean_entries = entries.unwrap();
        if clean_entries.version >= 2 {
            return Ok(Some(clean_entries));
        }
        clean_entries.data.iter_mut().for_each(|ent| {
            ent.id = 0;
        });
        clean_entries.version = 2;
        Ok(Some(clean_entries))
    }
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use super::Upgrade;

    mod v1 {
        use super::*;

        #[test]
        fn should_upgrade() {
            let content =
                "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
            let res = Upgrade::read_upgrade_to_v1(content);
            assert!(res.is_ok());
            let opt = res.unwrap();
            assert!(opt.is_some());
            let ent = opt.unwrap();
            assert_eq!(ent.data.len(), 2);
            assert_eq!(ent.version, 2)
        }

        #[test]
        fn should_not_upgrade_to_v1() {
            let content =
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";
            let res = Upgrade::read_upgrade_to_v1(content);
            assert!(res.is_ok());
            assert!(res.unwrap().is_none())
        }
    }

    mod v2 {
        use super::*;

        #[test]
        #[allow(deprecated)]
        fn should_upgrade() {
            let content =
                "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";
            let res = Upgrade::read_upgrade_to_v2(content);
            assert!(res.is_ok());
            let opt = res.unwrap();
            assert!(opt.is_some());
            let ent = opt.unwrap();
            assert_eq!(ent.data.len(), 2);
            assert_eq!(ent.data[0].id, 0);
            assert_eq!(ent.version, 2)
        }
    }

    #[test]
    fn should_chain_upgrade() {
        let content =
            "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
        let res = Upgrade::upgrade(content);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some());
        let ent = opt.unwrap();
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.version, 2)
    }

    #[test]
    fn should_panic_on_malformed() {
        let content =
            "{\"data\":[\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";
        let res = Upgrade::upgrade(content);
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_none())
    }
}
