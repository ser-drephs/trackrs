use std::{
    env,
    fs::{self, OpenOptions},
    io,
};

use clap::Parser;
use serial_test::serial;
use trackrs::{Cli, CliExecute, Settings};

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
fn start_break_continue_and_end_workflow(ctx: &mut IntegrationContext) {
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

    let c = Cli::parse_from(["trackrs", "continue"].iter());
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
fn start_break_continue_workflow(ctx: &mut IntegrationContext) {
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

    let c = Cli::parse_from(["trackrs", "continue"].iter());
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

    let c = Cli::parse_from(["trackrs", "continue"].iter());
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
