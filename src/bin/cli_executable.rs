use trackrs::TrackerError;

fn main() -> Result<(), TrackerError> {
    use clap::Parser;
    use trackrs::{Cli, CliExecute};

    let configuration = trackrs::Configuration::builder().add_defaults()?.add_json_source(&trackrs::Configuration::file())?.build()?;
    let storage = trackrs::JsonStorageProvider::new_today(configuration.folder.clone().into())?;

    let cli = Cli::parse();
    cli.init_logger()?;

    cli.execute(storage, configuration)
}
