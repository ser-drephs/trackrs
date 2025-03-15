pub(crate) fn setup() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_TEST", "true");

    env_logger::builder().is_test(true).try_init().unwrap()
}

pub const TERMINAL_GREEN: &str = "\u{1b}[92m";
pub const TERMINAL_RED: &str = "\u{1b}[91m";
pub const TERMINAL_NEUTRAL: &str = "\u{1b}[0m";
pub const TERMINAL_YELLOW: &str = "\u{1b}[93m";