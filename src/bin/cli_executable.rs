use trackrs::TrackerError;

fn main() -> Result<(), TrackerError> {
    use trackrs::{Cli, CliExecute};

    let cli = Cli::parse();
    cli.init_logger()?;

    cli.execute()
}
