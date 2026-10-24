use std::io;

use chrono::TimeZone;
use serde_json::Value;

use super::timesheet::CURRENT_VERSION;
use crate::{ Entry, Timesheet };

use super::UpgradeError;

pub struct Upgrade {}

impl Upgrade {
    /// Read entries of any known version and upgrade them to the current version. `zone` is the
    /// timezone the times of files before version 4 are written in.
    pub fn upgrade<R: io::Read, Tz: TimeZone>(
        reader: R,
        zone: &Tz
    ) -> Result<Timesheet, UpgradeError> {
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
        if timesheet.version == 3 {
            timesheet = Self::to_v4(timesheet, zone);
        }
        if timesheet.version != CURRENT_VERSION {
            return Err(UpgradeError::UnsupportedVersion(timesheet.version));
        }
        Ok(timesheet)
    }

    /// Versionless: a plain array of entries.
    fn to_v1(value: Value) -> Result<Timesheet, UpgradeError> {
        let data: Vec<Entry> = serde_json::from_value(value)?;
        Ok(Timesheet { data, version: 1, ..Default::default() })
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

    /// v4 stores the time of an entry as local time with its offset from UTC, instead of in UTC.
    /// The instant of an entry stays the same, it is written in the offset `zone` has at that
    /// moment.
    fn to_v4<Tz: TimeZone>(timesheet: Timesheet, zone: &Tz) -> Timesheet {
        let data = timesheet.data
            .into_iter()
            .map(|e| Entry { time: e.time.with_timezone(zone).fixed_offset(), ..e })
            .collect();
        Timesheet { data, version: 4 }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{ FixedOffset, Local, TimeZone, Utc };
    use serde_json::json;

    use super::{ Upgrade, UpgradeError };

    /// The timezone old files are read in, two hours ahead of UTC.
    fn zone() -> FixedOffset {
        FixedOffset::east_opt(2 * 3600).unwrap()
    }

    const VERSIONLESS: &str =
        "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-02T08:00:53.523332900Z\"}]";
    const V2: &str =
        "{\"data\":[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53Z\"},{\"id\":2,\"status\":\"Break\",\"time\":\"2022-02-02T04:00:53Z\",\"note\":\"lunch\"}],\"version\":2}";
    const V3: &str =
        "{\"data\":[{\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53Z\"},{\"status\":\"End\",\"time\":\"2022-07-02T08:00:53Z\"}],\"version\":3}";
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
    fn should_upgrade_v2_to_v3() {
        let v2 = serde_json::from_str(V2).unwrap();
        let ent = Upgrade::to_v3(v2);
        assert_eq!(ent.version, 3);
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.data[1].note.as_deref(), Some("lunch"));
    }

    #[test]
    fn should_upgrade_versionless_to_current() {
        let ent = Upgrade::upgrade(VERSIONLESS.as_bytes(), &zone()).unwrap();
        assert_eq!(ent.version, 4);
        assert_eq!(ent.data.len(), 2);
        assert!(ent.data.iter().all(|e| e.note.is_none()));
    }

    #[test]
    fn should_upgrade_v1_to_current() {
        let ent = Upgrade::upgrade(V1.as_bytes(), &zone()).unwrap();
        assert_eq!(ent.version, 4);
        assert_eq!(ent.data.len(), 2);
    }

    #[test]
    fn should_upgrade_v2_to_current() {
        let ent = Upgrade::upgrade(V2.as_bytes(), &zone()).unwrap();
        assert_eq!(ent.version, 4);
        assert_eq!(ent.data.len(), 2);
        assert_eq!(ent.data[1].note.as_deref(), Some("lunch"));
    }

    #[test]
    fn should_drop_ids_of_upgraded_versions() {
        for old in [VERSIONLESS, V1, V2] {
            let ent = Upgrade::upgrade(old.as_bytes(), &zone()).unwrap();
            let written = serde_json::to_string(&ent).unwrap();
            assert!(!written.contains("\"id\""), "{written}");
            assert!(written.ends_with("\"version\":4}"), "{written}");
        }
    }

    #[test]
    fn should_keep_current_version() {
        let content = json!({
            "data": [{"status": "Break", "time": "2022-02-02T00:00:53+05:30", "note": "lunch"}],
            "version": 4
        })
        .to_string();
        let ent = Upgrade::upgrade(content.as_bytes(), &zone()).unwrap();
        assert_eq!(ent.version, 4);
        assert_eq!(ent.data[0].note.as_deref(), Some("lunch"));
        // The offset of a current file is kept as it is.
        assert_eq!(5 * 3600 + 30 * 60, ent.data[0].time.offset().local_minus_utc());
    }

    #[test]
    fn should_upgrade_v3_to_v4() {
        let v3 = serde_json::from_str(V3).unwrap();
        let ent = Upgrade::to_v4(v3, &zone());

        assert_eq!(ent.version, 4);
        assert_eq!(ent.data.len(), 2);
    }

    #[test]
    fn should_keep_the_instants_and_write_them_in_the_given_zone_when_upgrading_v3() {
        let ent = Upgrade::upgrade(V3.as_bytes(), &zone()).unwrap();

        assert_eq!(ent.version, 4);
        // The same instants, now written two hours ahead of UTC.
        let written = serde_json::to_string(&ent).unwrap();
        assert!(written.contains("\"time\":\"2022-02-02T02:00:53+02:00\""), "{written}");
        assert!(written.contains("\"time\":\"2022-07-02T10:00:53+02:00\""), "{written}");
        let utc = [
            Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 53).unwrap(),
            Utc.with_ymd_and_hms(2022, 7, 2, 8, 0, 53).unwrap(),
        ];
        for (entry, utc) in ent.data.iter().zip(utc) {
            assert_eq!(utc, entry.time);
            assert_eq!(2 * 3600, entry.time.offset().local_minus_utc());
        }
    }

    #[test]
    fn should_write_old_versions_in_the_given_zone() {
        for old in [VERSIONLESS, V1, V2, V3] {
            let ent = Upgrade::upgrade(old.as_bytes(), &zone()).unwrap();

            assert_eq!(ent.version, 4);
            for entry in &ent.data {
                assert_eq!(2 * 3600, entry.time.offset().local_minus_utc(), "{old}");
            }
        }
    }

    #[test]
    fn should_follow_the_offset_a_zone_has_at_the_moment_of_each_entry() {
        // Whatever the offsets of the machine are in February and July.
        let ent = Upgrade::upgrade(V3.as_bytes(), &Local).unwrap();

        let utc = [
            Utc.with_ymd_and_hms(2022, 2, 2, 0, 0, 53).unwrap(),
            Utc.with_ymd_and_hms(2022, 7, 2, 8, 0, 53).unwrap(),
        ];
        for (entry, utc) in ent.data.iter().zip(utc) {
            assert_eq!(utc, entry.time);
            let local = Local.offset_from_utc_datetime(&utc.naive_utc());
            assert_eq!(local, *entry.time.offset());
        }
    }

    #[test]
    fn should_reject_version_zero() {
        let res = Upgrade::upgrade("{\"data\":[],\"version\":0}".as_bytes(), &zone());
        assert!(matches!(res, Err(UpgradeError::UnsupportedVersion(0))));
    }

    #[test]
    fn should_reject_unknown_version() {
        let content = "{\"data\":[],\"version\":99}";
        let res = Upgrade::upgrade(content.as_bytes(), &zone());
        assert!(matches!(res, Err(UpgradeError::UnsupportedVersion(99))));
    }

    #[test]
    fn should_fail_on_malformed() {
        let content =
            "{\"data\":[\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-02T00:00:53.523319900Z\"}],\"version\":1}";
        let res = Upgrade::upgrade(content.as_bytes(), &zone());
        assert!(matches!(res, Err(UpgradeError::ReadError(_))));
    }
}
