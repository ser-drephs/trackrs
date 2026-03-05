#[allow(dead_code)]
pub(crate) fn setup() {
    std::env::set_var("RUST_TEST", "true");
    logger();
}

#[allow(dead_code)]
pub(crate) fn logger() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::builder().is_test(true).try_init().unwrap()
}
#[allow(dead_code)]
pub const TERMINAL_GREEN: &str = "\u{1b}[92m";
#[allow(dead_code)]
pub const TERMINAL_RED: &str = "\u{1b}[91m";
#[allow(dead_code)]
pub const TERMINAL_NEUTRAL: &str = "\u{1b}[0m";
#[allow(dead_code)]
pub const TERMINAL_YELLOW: &str = "\u{1b}[93m";
