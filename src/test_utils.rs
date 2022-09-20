use std::process::Command;

use crate::TrackerError;

pub(crate) type TestResult = std::result::Result<(), TrackerError>;

pub fn init() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_TEST", "true");
    let _ = env_logger::builder().is_test(true).try_init();
}

/// todo: remove
pub fn wait(seconds: u8) {
    let mut child = Command::new("sleep").arg(format!("{}", seconds)).spawn().unwrap();
    let _result = child.wait().unwrap();
}
