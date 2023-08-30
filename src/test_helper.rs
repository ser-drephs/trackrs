use std::{fs::File, io::Write, process::Command};

use tempfile::TempDir;

pub fn setup() {
    std::env::set_var("RUST_TEST", "true");
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn wait() {
    let mut child = Command::new("sleep").arg("1").spawn().unwrap();
    let _result = child.wait().unwrap();
}

pub fn write_test_file(folder: &TempDir, name: &str, content: &str) -> Result<(), std::io::Error> {
    let file_content = "[{\"id\":1,\"status\":\"Connect\",\"time\":\"2022-08-04T23:00:53.523319900Z\"},{\"id\":2,\"status\":\"End\",\"time\":\"2022-08-04T23:00:53.523332900Z\"}]";
    let time_file = folder.path().join("20220804.json");
    let mut file = File::create(time_file)?;
    file.write_all(file_content.as_bytes())?;
    self::wait();
    Ok(())
}
