use std::panic;
use std::process::Command;

pub fn wait(seconds: u8) {
    let mut child = Command::new("sleep")
        .arg(format!("{}", seconds))
        .spawn()
        .unwrap();
    let _result = child.wait().unwrap();
}

pub fn setup() {
    std::env::set_var("RUST_LOG", "trace");
    let _ = env_logger::builder().is_test(true).try_init();
}

pub fn catch_unwind_silent<F: FnOnce() -> R + panic::UnwindSafe, R>(f: F) -> std::thread::Result<R> {
    let prev_hook = panic::take_hook();
    panic::set_hook(Box::new(|_| {}));
    let result = panic::catch_unwind(f);
    panic::set_hook(prev_hook);
    result
}
