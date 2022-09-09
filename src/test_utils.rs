use std::process::Command;

pub fn init() {
    std::env::set_var("RUST_LOG", "debug");
    let _ = env_logger::builder().is_test(true).try_init();
}

/// todo: remove
pub fn wait(seconds: u8) {
    let mut child = Command::new("sleep").arg(format!("{}", seconds)).spawn().unwrap();
    let _result = child.wait().unwrap();
}
