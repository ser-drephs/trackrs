use trackrs::TrackerError;

use clap::parser;

#[cfg(not(tarpaulin_include))]
fn main() {//-> Result<(), TrackerError> {
    use clap::{arg, command, Command};
    //use trackrs::{Cli, CliExecute};

    let matches = command!()
    .arg(arg!(-v --verbose ... "Verbose logging"))
    .subcommand(trackrs::start_cmd()).get_matches();
    //let cli = Cli::parse();
    //cli.init_logger()?;

    //cli.execute()
}
