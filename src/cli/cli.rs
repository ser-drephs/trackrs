/// Simple time tracker using CLI.
///
/// A simple time tracker using the CLI. Writes an entry with the current timestamp for each command that is invoked.
#[derive(Parser, Debug)]
#[clap(author, version, about)]
#[clap(propagate_version = true)]
pub struct Cli {
    #[clap(subcommand)]
    command: Commands,

    #[clap(flatten)]
    verbose: clap_verbosity_flag::Verbosity,
}

pub trait Cli {
    fn execute(&self) -> TrackerResult;
    fn init_logger(&self) -> TrackerResult;
}
