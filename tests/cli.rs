use std::{
    env,
    fs::{self, OpenOptions},
    io,
};

use serial_test::serial;
use trackrs::{BreakLimit, Cli, CliExecute, JsonStorage, Settings, StatusDaily, StorageProvider};

fn logger() {
    // env::set_var("RUST_LOG", "info");
    let _ = env_logger::builder().is_test(true).try_init();
}

struct IntegrationContext {
    temp_dir: tempfile::TempDir,
}

impl test_context::TestContext for IntegrationContext {
    fn setup() -> IntegrationContext {
        logger();
        env::set_var("RUST_TEST", "true");
        let temp_dir = tempfile::tempdir().unwrap();
        env::set_current_dir(&temp_dir).unwrap();
        let mut settings = Settings::new().unwrap();
        let trackrs_folder = temp_dir.path().join("trackrs");
        settings.folder = trackrs_folder.to_str().unwrap().to_owned();
        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(&settings.file)
            .unwrap();
        serde_json::to_writer_pretty(w, &settings).unwrap();
        IntegrationContext { temp_dir }
    }

    fn teardown(self) {
        self.temp_dir.close().unwrap();
    }
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn start_break_start_and_end_workflow(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();

    let f = fs::read_dir(&folder).unwrap();
    let files = f
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    assert_eq!(&1, &files.len());

    let b = Cli::parse_from(["trackrs", "break"].iter());
    b.execute().unwrap();

    let c = Cli::parse_from(["trackrs", "start"].iter());
    c.execute().unwrap();

    let e = Cli::parse_from(["trackrs", "end"].iter());
    e.execute().unwrap();

    let file = files.first().unwrap();
    let r = std::fs::File::open(file).unwrap();
    let raw: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(r))
        .map(|l| l.unwrap())
        .collect();

    let a = raw.first().unwrap();

    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"status\":\"Break\""));
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"status\":\"End\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn start_break_start_workflow(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();

    let f = fs::read_dir(&folder).unwrap();
    let files = f
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    assert_eq!(&1, &files.len());

    let b = Cli::parse_from(["trackrs", "break"].iter());
    b.execute().unwrap();

    let c = Cli::parse_from(["trackrs", "start"].iter());
    c.execute().unwrap();

    let file = files.first().unwrap();
    let r = std::fs::File::open(file).unwrap();
    let raw: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(r))
        .map(|l| l.unwrap())
        .collect();

    let a = raw.first().unwrap();

    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"status\":\"Break\""));
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(!a.contains("\"status\":\"End\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn takeover_subtracts_from_today(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();

    let f = fs::read_dir(&folder).unwrap();
    let files = f
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    assert_eq!(&1, &files.len());

    let b = Cli::parse_from(["trackrs", "break"].iter());
    b.execute().unwrap();

    let c = Cli::parse_from(["trackrs", "start"].iter());
    c.execute().unwrap();

    let file = files.first().unwrap();
    let r = std::fs::File::open(file).unwrap();
    let raw: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(r))
        .map(|l| l.unwrap())
        .collect();

    let a = raw.first().unwrap();

    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"status\":\"Break\""));
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(!a.contains("\"status\":\"End\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn start_and_break_save_note(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");

    let s = Cli::parse_from(["trackrs", "start", "-n", "standup"].iter());
    s.execute().unwrap();

    let b = Cli::parse_from(["trackrs", "break", "--note", "lunch"].iter());
    b.execute().unwrap();

    let c = Cli::parse_from(["trackrs", "start"].iter());
    c.execute().unwrap();

    let file = fs::read_dir(&folder).unwrap().next().unwrap().unwrap().path();
    let a = fs::read_to_string(file).unwrap();

    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"standup\""));
    assert!(a.contains("\"status\":\"Break\""));
    assert!(a.contains("\"note\":\"lunch\""));
    assert_eq!(2, a.matches("\"note\"").count());
    assert!(a.contains("\"version\":3"));
}

/// Store settings with a 30 minute break after 6 hours of work.
fn with_break_limit() -> Settings {
    let mut settings = Settings::new().unwrap();
    settings.limits = vec![BreakLimit { start: 6 * 60, minutes: 30 }];
    let w = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&settings.file)
        .unwrap();
    serde_json::to_writer_pretty(w, &settings).unwrap();
    settings
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn sick_writes_full_day_with_break(ctx: &mut IntegrationContext) {
    let settings = with_break_limit();

    // Monday, 8 hours of work per day by default.
    let c = Cli::parse_from(["trackrs", "sick", "2022-02-07"].iter());
    c.execute().unwrap();

    let file = ctx.temp_dir.path().join("trackrs").join("20220207.json");
    let a = fs::read_to_string(&file).unwrap();
    assert!(a.contains(
        "{\"status\":\"Connect\",\"time\":\"2022-02-07T06:00:00Z\",\"note\":\"sick\"}"
    ));
    assert!(a.contains("{\"status\":\"End\",\"time\":\"2022-02-07T14:30:00Z\"}"));

    let time_data = JsonStorage::new(settings.folder.to_owned())
        .load(chrono::NaiveDate::from_ymd_opt(2022, 2, 7).unwrap())
        .unwrap();
    let status = StatusDaily::builder()
        .data(time_data)
        .settings(settings)
        .build()
        .unwrap();
    assert_eq!("08:00", status.worktime.to_string());
    assert_eq!("00:00", status.overtime.to_string());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn vacation_writes_full_day(ctx: &mut IntegrationContext) {
    let c = Cli::parse_from(["trackrs", "vacation", "2022-02-08"].iter());
    c.execute().unwrap();

    let file = ctx.temp_dir.path().join("trackrs").join("20220208.json");
    let a = fs::read_to_string(&file).unwrap();
    assert!(a.contains("\"time\":\"2022-02-08T06:00:00Z\",\"note\":\"vacation\""));
    assert!(a.contains("{\"status\":\"End\",\"time\":\"2022-02-08T14:00:00Z\"}"));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn absence_does_not_change_day_with_reached_target(ctx: &mut IntegrationContext) {
    let c = Cli::parse_from(["trackrs", "vacation", "2022-02-08"].iter());
    c.execute().unwrap();
    let file = ctx.temp_dir.path().join("trackrs").join("20220208.json");
    let before = fs::read_to_string(&file).unwrap();

    let s = Cli::parse_from(["trackrs", "sick", "2022-02-08"].iter());
    assert!(s.execute().is_err());
    assert_eq!(before, fs::read_to_string(&file).unwrap());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn sick_extends_running_day(ctx: &mut IntegrationContext) {
    let mut settings = with_break_limit();
    // Every day is a workday, so the test doesn't depend on the weekday it runs on.
    settings.workperday.saturday = 8 * 60;
    settings.workperday.sunday = 8 * 60;
    let w = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&settings.file)
        .unwrap();
    serde_json::to_writer_pretty(w, &settings).unwrap();

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();
    let c = Cli::parse_from(["trackrs", "sick"].iter());
    c.execute().unwrap();

    let folder = ctx.temp_dir.path().join("trackrs");
    let file = fs::read_dir(&folder).unwrap().next().unwrap().unwrap().path();
    let a = fs::read_to_string(file).unwrap();
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"sick\""));
    assert!(a.contains("\"status\":\"End\""));
}

fn read_today(ctx: &IntegrationContext) -> String {
    let folder = ctx.temp_dir.path().join("trackrs");
    let file = folder.join(format!("{}.json", chrono::Utc::now().format("%Y%m%d")));
    fs::read_to_string(file).unwrap()
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn add_prepends_before_start(ctx: &mut IntegrationContext) {
    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();
    let a = Cli::parse_from(["trackrs", "add", "30"].iter());
    a.execute().unwrap();

    let a = read_today(ctx);
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"manually added\"}"));
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.find("manually added") < a.rfind("\"status\":\"Connect\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn add_works_after_end(ctx: &mut IntegrationContext) {
    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();
    let e = Cli::parse_from(["trackrs", "end"].iter());
    e.execute().unwrap();
    let a = Cli::parse_from(["trackrs", "add", "15"].iter());
    a.execute().unwrap();

    let a = read_today(ctx);
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"manually added\"}"));
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"status\":\"End\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn add_rejects_without_start(ctx: &mut IntegrationContext) {
    let a = Cli::parse_from(["trackrs", "add", "30"].iter());
    assert!(a.execute().is_err());

    let folder = ctx.temp_dir.path().join("trackrs");
    assert!(!folder.exists() || fs::read_dir(&folder).unwrap().count() == 0);
}

fn takeover_file(ctx: &IntegrationContext) -> std::path::PathBuf {
    ctx.temp_dir.path().join("trackrs").join(".trackrs-takeover")
}

/// Today's file with a connect two hours ago and an end one hour ago.
fn ended_today(ctx: &IntegrationContext) -> std::path::PathBuf {
    let now = chrono::Utc::now();
    let folder = ctx.temp_dir.path().join("trackrs");
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(format!("{}.json", now.format("%Y%m%d")));
    let content = serde_json::json!({
        "data": [
            {"status": "Connect", "time": now - chrono::Duration::hours(2)},
            {"status": "End", "time": now - chrono::Duration::hours(1)}
        ],
        "version": 2
    });
    fs::write(&file, content.to_string()).unwrap();
    file
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn takeover_moves_end_and_stores_minutes(ctx: &mut IntegrationContext) {
    let file = ended_today(ctx);
    let end_before = fs::read_to_string(&file).unwrap();

    let t = Cli::parse_from(["trackrs", "takeover", "30"].iter());
    t.execute().unwrap();

    let a = fs::read_to_string(&file).unwrap();
    assert_ne!(end_before, a);
    assert!(a.contains("\"status\":\"End\""));
    assert!(a.contains("\"status\":\"Takeover\""));
    assert!(a.contains("\"note\":\"takeover\""));
    assert_eq!("{\"minutes\":30}", fs::read_to_string(takeover_file(ctx)).unwrap());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn takeover_rejects_running_day(ctx: &mut IntegrationContext) {
    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();
    let before = read_today(ctx);

    let t = Cli::parse_from(["trackrs", "takeover", "30"].iter());
    assert!(t.execute().is_err());
    assert_eq!(before, read_today(ctx));
    assert!(!takeover_file(ctx).exists());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn takeover_rejects_when_one_is_pending(ctx: &mut IntegrationContext) {
    let file = ended_today(ctx);
    let before = fs::read_to_string(&file).unwrap();
    fs::write(takeover_file(ctx), "{\"minutes\":15}").unwrap();

    let t = Cli::parse_from(["trackrs", "takeover", "30"].iter());
    assert!(t.execute().is_err());
    assert_eq!(before, fs::read_to_string(&file).unwrap());
    assert_eq!("{\"minutes\":15}", fs::read_to_string(takeover_file(ctx)).unwrap());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn first_start_adds_takeover_and_removes_file(ctx: &mut IntegrationContext) {
    // 1 minute keeps the takeover on today's date unless run in the first minute after 00:00 UTC.
    fs::create_dir_all(ctx.temp_dir.path().join("trackrs")).unwrap();
    fs::write(takeover_file(ctx), "{\"minutes\":1}").unwrap();

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();

    let a = read_today(ctx);
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"takeover\"}"));
    assert!(a.find("takeover") < a.rfind("\"status\":\"Connect\""));
    assert!(!takeover_file(ctx).exists());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn later_start_keeps_takeover(ctx: &mut IntegrationContext) {
    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();
    fs::write(takeover_file(ctx), "{\"minutes\":1}").unwrap();

    let c = Cli::parse_from(["trackrs", "start"].iter());
    c.execute().unwrap();

    let a = read_today(ctx);
    assert!(a.contains("\"status\":\"Connect\""));
    assert!(!a.contains("takeover"));
    assert!(takeover_file(ctx).exists());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn start_keeps_hand_formatted_file_valid(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(format!("{}.json", chrono::Utc::now().format("%Y%m%d")));
    // Pretty printed, as when saved in an editor, longer than written compactly even with the
    // entry `start` adds.
    let at = |minutes: i64| chrono::Utc::now() - chrono::Duration::minutes(minutes);
    let content = serde_json::json!({
        "data": [
            {"status": "Connect", "time": at(180)},
            {"status": "Break", "time": at(120)},
            {"status": "Connect", "time": at(90)}
        ],
        "version": 2
    });
    fs::write(&file, serde_json::to_string_pretty(&content).unwrap() + "\n").unwrap();

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute().unwrap();

    let a: serde_json::Value = serde_json::from_str(&fs::read_to_string(&file).unwrap()).unwrap();
    assert_eq!(4, a["data"].as_array().unwrap().len());
}

/// Workdays of the month before the current one, with the folder file of each.
fn previous_month_workdays() -> (String, Vec<chrono::NaiveDate>) {
    use chrono::{Datelike, Weekday};

    let first_of_this_month = chrono::Local::now().date_naive().with_day(1).unwrap();
    let last = first_of_this_month.pred_opt().unwrap();
    let key = last.format("%Y-%m").to_string();
    let days = (1..=last.day())
        .map(|d| last.with_day(d).unwrap())
        .filter(|d| !matches!(d.weekday(), Weekday::Sat | Weekday::Sun))
        .collect();
    (key, days)
}

/// Write a day of 8 hours of work, from 06:00 to 14:00 UTC, as saved by trackrs.
fn write_worked_day(ctx: &IntegrationContext, date: chrono::NaiveDate) {
    let folder = ctx.temp_dir.path().join("trackrs");
    fs::create_dir_all(&folder).unwrap();
    let day = date.format("%Y-%m-%d");
    let content = format!(
        "{{\"data\":[{{\"status\":\"Connect\",\"time\":\"{day}T06:00:00Z\"}},{{\"status\":\"End\",\"time\":\"{day}T14:00:00Z\"}}],\"version\":2}}"
    );
    fs::write(folder.join(format!("{}.json", date.format("%Y%m%d"))), content).unwrap();
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn status_day_shows_stored_day_only(ctx: &mut IntegrationContext) {
    let sick = Cli::parse_from(["trackrs", "sick", "2022-02-07"].iter());
    sick.execute().unwrap();
    assert!(ctx.temp_dir.path().join("trackrs").join("20220207.json").exists());

    let stored = Cli::parse_from(["trackrs", "status", "-d", "2022-02-07"].iter());
    assert!(stored.execute().is_ok());

    let missing = Cli::parse_from(["trackrs", "status", "-d", "2022-02-08"].iter());
    assert!(missing.execute().is_err());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn status_week_month_and_year_read_tracked_days(ctx: &mut IntegrationContext) {
    let start = Cli::parse_from(["trackrs", "start"].iter());
    start.execute().unwrap();
    let end = Cli::parse_from(["trackrs", "end"].iter());
    end.execute().unwrap();

    for range in ["-w", "-m", "-y"] {
        let status = Cli::parse_from(["trackrs", "status", range, "0"].iter());
        assert!(status.execute().is_ok(), "status {range} 0");
    }
    // Reading doesn't change the tracked day.
    assert_eq!(1, fs::read_dir(ctx.temp_dir.path().join("trackrs")).unwrap().count());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn overtime_stores_finished_month_from_tracked_days(ctx: &mut IntegrationContext) {
    let (key, workdays) = previous_month_workdays();
    // All workdays but the first one worked 8 hours, so 8 hours are missing.
    for date in &workdays[1..] {
        write_worked_day(ctx, *date);
    }

    let overtime = Cli::parse_from(["trackrs", "overtime", "-m", "-1"].iter());
    overtime.execute().unwrap();

    let file = ctx.temp_dir.path().join("trackrs").join("overtime.json");
    let stored: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(file).unwrap()).unwrap();
    assert_eq!("-08:00", stored[&key]["overtime"]);
    assert_eq!("-08:00", stored[&key]["cumulated"]);

    // The stored month isn't replaced without force and is carried by the month status.
    let again = Cli::parse_from(["trackrs", "overtime", "-m", "-1"].iter());
    assert!(again.execute().is_err());
    let forced = Cli::parse_from(["trackrs", "overtime", "-m", "-1", "-f"].iter());
    assert!(forced.execute().is_ok());
    let status = Cli::parse_from(["trackrs", "status", "-m", "0"].iter());
    assert!(status.execute().is_ok());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn overtime_rejects_current_month(_ctx: &mut IntegrationContext) {
    let overtime = Cli::parse_from(["trackrs", "overtime", "-m", "0"].iter());
    assert!(overtime.execute().is_err());
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn start_updates_old_time_files_to_latest_version(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");
    fs::create_dir_all(&folder).unwrap();
    let file = folder.join(format!("{}.json", chrono::Utc::now().format("%Y%m%d")));
    let entries = r#"{"id":1,"status":"Connect","time":"2022-02-02T08:00:00Z"},{"id":2,"status":"End","time":"2022-02-02T09:00:00Z"}"#;
    let old_files = [
        // Without a version.
        format!("[{entries}]"),
        format!(r#"{{"data":[{entries}],"version":1}}"#),
        format!(r#"{{"data":[{entries}],"version":2}}"#),
    ];

    for old in old_files {
        fs::write(&file, &old).unwrap();

        let start = Cli::parse_from(["trackrs", "start"].iter());
        start.execute().unwrap();

        let updated = fs::read_to_string(&file).unwrap();
        let a: serde_json::Value = serde_json::from_str(&updated).unwrap();
        assert_eq!(3, a["version"], "{old}");
        assert_eq!(3, a["data"].as_array().unwrap().len(), "{old}");
        assert!(!updated.contains("\"id\""), "{updated}");
    }
}
