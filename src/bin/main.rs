use trackrs::{ cli, storage, config };

// #[cfg(not(tarpaulin_include))]
fn main() {
    let storage_provider = storage::JsonStorageProvider::new_today().unwrap();
    let config_provider = config::JsonConfigurationProvider::new();

    let commands = cli::commands();
    let matches = commands.get_matches();

    cli::init_logger(&matches);

    cli::execute(&matches, &storage_provider, &config_provider).expect("error during execution")
}
