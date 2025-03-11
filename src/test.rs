pub(crate) fn setup() {
    std::env::set_var("RUST_LOG", "debug");
    std::env::set_var("RUST_TEST", "true");

    env_logger::builder().is_test(true).try_init().unwrap()
}
