use trackrs::config::Configuration;
use trackrs::json_storage_provider::JsonStorageProvider;
use trackrs::TrackerError;

fn main() -> Result<(), TrackerError> {
    use clap::Parser;
    use trackrs::{Cli, CliExecute};
    let configuration = Configuration::builder()
        .add_defaults()?
        .add_json_source(&Configuration::file())?
        .build()?;
    let storage = JsonStorageProvider::new_today(configuration.folder.clone().into())?;

    let cli = Cli::parse();
    cli.init_logger()?;

    cli.execute(&storage, &configuration)
}
