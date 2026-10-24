use std::{
    env,
    fs::{self, OpenOptions},
    io,
};

use serial_test::serial;
use chrono::TimeZone;
use trackrs::{BreakLimit, Cli, CliExecute, Settings, StatusDaily, TimeData};

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

    assert!(a.contains("\"id\":1,\"status\":\"Connect\""));
    assert!(a.contains("\"id\":2,\"status\":\"Break\""));
    assert!(a.contains("\"id\":3,\"status\":\"Connect\""));
    assert!(a.contains("\"id\":4,\"status\":\"End\""));
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

    assert!(a.contains("\"id\":1,\"status\":\"Connect\""));
    assert!(a.contains("\"id\":2,\"status\":\"Break\""));
    assert!(a.contains("\"id\":3,\"status\":\"Connect\""));
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

    assert!(a.contains("\"id\":1,\"status\":\"Connect\""));
    assert!(a.contains("\"id\":2,\"status\":\"Break\""));
    assert!(a.contains("\"id\":3,\"status\":\"Connect\""));
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

    assert!(a.contains("\"id\":1,\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"standup\""));
    assert!(a.contains("\"id\":2,\"status\":\"Break\""));
    assert!(a.contains("\"note\":\"lunch\""));
    assert_eq!(2, a.matches("\"note\"").count());
    assert!(a.contains("\"version\":2"));
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
        "{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-02-07T06:00:00Z\",\"note\":\"sick\"}"
    ));
    assert!(a.contains("{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-07T14:30:00Z\"}"));

    let mut time_data = TimeData::builder()
        .folder(settings.folder.to_owned().into())
        .date(chrono::Utc.with_ymd_and_hms(2022, 2, 7, 6, 0, 0).unwrap())
        .build()
        .unwrap();
    time_data.read_from_file().unwrap();
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
    assert!(a.contains("{\"id\":2,\"status\":\"End\",\"time\":\"2022-02-08T14:00:00Z\"}"));
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
    assert!(a.contains("\"id\":1,\"status\":\"Connect\""));
    assert!(a.contains("\"id\":2,\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"sick\""));
    assert!(a.contains("\"id\":3,\"status\":\"End\""));
}

fn read_today(ctx: &IntegrationContext) -> String {
    let folder = ctx.temp_dir.path().join("trackrs");
    let file = fs::read_dir(&folder).unwrap().next().unwrap().unwrap().path();
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
    assert!(a.contains("\"id\":1,\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"manually added\"}"));
    assert!(a.contains("\"id\":2,\"status\":\"Connect\""));
    assert!(a.find("manually added") < a.find("\"id\":2"));
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
    assert!(a.contains("\"id\":1,\"status\":\"Connect\""));
    assert!(a.contains("\"note\":\"manually added\"}"));
    assert!(a.contains("\"id\":2,\"status\":\"Connect\""));
    assert!(a.contains("\"id\":3,\"status\":\"End\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn add_rejects_without_start(ctx: &mut IntegrationContext) {
    let a = Cli::parse_from(["trackrs", "add", "30"].iter());
    assert!(a.execute().is_err());

    let folder = ctx.temp_dir.path().join("trackrs");
    assert_eq!(0, fs::read_dir(&folder).unwrap().count());
}
