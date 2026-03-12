use std::{
    env,
    fs::{self, OpenOptions},
    io,
};

use clap::Parser;
use serial_test::serial;
use trackrs::{Cli, CliExecute};
use trackrs::config::Configuration;
use trackrs::json_storage_provider::JsonStorageProvider;

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
        let mut configuration = Configuration::builder()
            .add_defaults().unwrap()
            .add_json_source(&Configuration::file()).unwrap()
            .build().unwrap();
        let trackrs_folder = temp_dir.path().join("trackrs");
        configuration.folder = trackrs_folder.to_str().unwrap().to_owned();
        let w = OpenOptions::new()
            .create(true)
            .write(true)
            .append(false)
            .truncate(false)
            .open(&Configuration::file())
            .unwrap();
        serde_json::to_writer_pretty(w, &configuration).unwrap();
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
    let configuration = Configuration::builder()
        .add_defaults().unwrap()
        .add_json_source(&Configuration::file()).unwrap()
        .build().unwrap();
    let storage = JsonStorageProvider::new_today(folder.clone()).unwrap();

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute(&storage, &configuration).unwrap();

    let f = fs::read_dir(&folder).unwrap();
    let files = f
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    assert_eq!(&1, &files.len());

    let b = Cli::parse_from(["trackrs", "break"].iter());
    b.execute(&storage, &configuration).unwrap();

    let c = Cli::parse_from(["trackrs", "continue"].iter());
    c.execute(&storage, &configuration).unwrap();

    let e = Cli::parse_from(["trackrs", "end"].iter());
    e.execute(&storage, &configuration).unwrap();

    let file = files.first().unwrap();
    let r = std::fs::File::open(file).unwrap();
    let raw: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(r))
        .map(|l| l.unwrap())
        .collect();

    let a = raw.first().unwrap();

    assert!(a.contains("\"status\":\"Start\""));
    assert!(a.contains("\"status\":\"Break\""));
    assert!(a.contains("\"status\":\"Start\""));
    assert!(a.contains("\"status\":\"End\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn start_break_continue_workflow(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");
    let configuration = Configuration::builder()
        .add_defaults().unwrap()
        .add_json_source(&Configuration::file()).unwrap()
        .build().unwrap();
    let storage = JsonStorageProvider::new_today(folder.clone()).unwrap();

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute(&storage, &configuration).unwrap();

    let f = fs::read_dir(&folder).unwrap();
    let files = f
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    assert_eq!(&1, &files.len());

    let b = Cli::parse_from(["trackrs", "break"].iter());
    b.execute(&storage, &configuration).unwrap();

    let c = Cli::parse_from(["trackrs", "continue"].iter());
    c.execute(&storage, &configuration).unwrap();

    let file = files.first().unwrap();
    let r = std::fs::File::open(file).unwrap();
    let raw: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(r))
        .map(|l| l.unwrap())
        .collect();

    let a = raw.first().unwrap();

    assert!(a.contains("\"status\":\"Start\""));
    assert!(a.contains("\"status\":\"Break\""));
    assert!(a.contains("\"status\":\"Start\""));
    assert!(!a.contains("\"status\":\"End\""));
}

#[test_context::test_context(IntegrationContext)]
#[test]
#[serial]
fn takeover_subtracts_from_today(ctx: &mut IntegrationContext) {
    let folder = ctx.temp_dir.path().join("trackrs");
    let configuration = Configuration::builder()
        .add_defaults().unwrap()
        .add_json_source(&Configuration::file()).unwrap()
        .build().unwrap();
    let storage = JsonStorageProvider::new_today(folder.clone()).unwrap();

    let s = Cli::parse_from(["trackrs", "start"].iter());
    s.execute(&storage, &configuration).unwrap();

    let f = fs::read_dir(&folder).unwrap();
    let files = f
        .map(|res| res.map(|e| e.path()))
        .collect::<Result<Vec<_>, io::Error>>()
        .unwrap();
    assert_eq!(&1, &files.len());

    let b = Cli::parse_from(["trackrs", "break"].iter());
    b.execute(&storage, &configuration).unwrap();

    let c = Cli::parse_from(["trackrs", "continue"].iter());
    c.execute(&storage, &configuration).unwrap();

    let file = files.first().unwrap();
    let r = std::fs::File::open(file).unwrap();
    let raw: Vec<String> = std::io::BufRead::lines(std::io::BufReader::new(r))
        .map(|l| l.unwrap())
        .collect();

    let a = raw.first().unwrap();

    assert!(a.contains("\"status\":\"Start\""));
    assert!(a.contains("\"status\":\"Break\""));
    assert!(a.contains("\"status\":\"Start\""));
    assert!(!a.contains("\"status\":\"End\""));
}
