use trackrs::{ cli, storage, config };

// #[cfg(not(tarpaulin_include))]
fn main() {
    config::ConfigurationFile::verify().expect("configuration file can not be accessed");

    let storage_provider = storage::JsonStorageProvider::new_today().unwrap();
    let config = config::Configuration::new().unwrap();
    let commands = cli::commands();
    let matches = commands.get_matches();

    cli::init_logger(&matches);

    cli::execute(&matches, &storage_provider, &config).expect("error during execution")
}
