use std::io;

use crate::{ Entries, Entry };

use super::UpgradeError;

pub struct Upgrade {}

impl Upgrade {
    pub fn to_v1<R: io::Read>(reader: R) -> Result<Option<Entries>, UpgradeError> {
        let vec_entries: Result<Vec<Entry>, serde_json::Error> = serde_json::from_reader(reader);
        if vec_entries.is_err() {
            let err = vec_entries.unwrap_err();
            if err.is_data() || err.is_eof() {
                return Ok(None);
            }
            return Err(UpgradeError::UpgradeV1Error(err));
        }

        let mut entries = Entries::new();
        entries.append(&mut vec_entries.unwrap());
        Ok(Some(entries))
    }
}

#[cfg(test)]
mod tests {
    use super::Upgrade;

    #[test]
    fn should_upgrade_to_v1() {
        let content =
            "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
        let res = Upgrade::to_v1(content.as_bytes());
        assert!(res.is_ok());
        let opt = res.unwrap();
        assert!(opt.is_some());
        let ent = opt.unwrap();
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.version, 1)
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
