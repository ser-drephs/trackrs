use std::io;

use crate::{ Timesheet, Entry };
#[allow(deprecated)]
use crate::EntryV1;

use super::UpgradeError;

pub struct Upgrade {}

impl Upgrade {
    pub fn upgrade<R: io::Read>(reader: R) -> Result<Option<Timesheet>, UpgradeError> {
        Upgrade::to_v1(reader)
    }

    #[allow(deprecated)]
    fn to_v1<R: io::Read>(reader: R) -> Result<Option<Timesheet>, UpgradeError> {
        let vec_entries: Result<Vec<EntryV1>, serde_json::Error> = serde_json::from_reader(reader);
        if vec_entries.is_err() {
            let err = vec_entries.unwrap_err();
            if err.is_data() {
                return Ok(None);
            }
            log::debug!("{:?}", err);
            return Err(UpgradeError::UpgradeV1Error(err));
        }

        let mut entries = Timesheet::new();
        let mut map: Vec<Entry> = vec_entries
            .unwrap()
            .into_iter()
            .map(|f| f.upgrade())
            .collect();
        entries.append(&mut map);
        Ok(Some(entries))
    }
}

#[cfg(test)]
mod tests {
    use crate::test;

    use super::Upgrade;

    #[test]
    fn should_upgrade_to_v1() {
        test::setup();
        let content =
            "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
        let res = Upgrade::to_v1(content.as_bytes());
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some());
        let ent = opt.unwrap();
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.version(), 1)
    }

    #[test]
    fn should_not_upgrade_to_v1() {
        let content =
            "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";
        let res = Upgrade::to_v1(content.as_bytes());
        assert!(res.is_ok());
        assert!(res.unwrap().is_none())
    }

    #[test]
    fn should_panic_on_malformed() {
        let content =
            "{\"data\":[\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}],\"version\":1}";
        let res = Upgrade::to_v1(content.as_bytes());
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_none())
    }
}
