use log::LevelFilter;
use trackrs::cli;

use clap::{command, Arg, ArgAction};
use trackrs::providers;

// #[cfg(not(tarpaulin_include))]
fn main() {
    let provider = providers::JsonProvider::new_today().unwrap();

    let matches = command!()
        .subcommand_required(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .help("Verbose logging")
                .action(ArgAction::Count)
                .global(true),
        )
        .subcommand(cli::start_cmd())
        .get_matches();

    init_logger(matches.get_count("verbose"));

    match matches.subcommand() {
        Some((cli::START_CMD, _sub_matches)) => cli::start_action(&provider).unwrap(),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
    }
}

fn init_logger(v: u8) {
    let verbosity = match v {
        0 => log::Level::Warn,
        1 => log::Level::Info,
        2 => log::Level::Debug,
        _ => log::Level::Trace,
    };

    let mut builder = env_logger::builder();
    let logger = builder
        .filter_level(verbosity.to_level_filter())
        .format_target(false);

    if verbosity >= LevelFilter::Debug {
        logger.format_target(true);
    }

    let err = logger.try_init();
    if err.is_err() {
        eprintln!("{:?}", err.unwrap_err());
    }

    log::info!("Informational logging is active.");
    log::debug!("Debug logging is active.");
    log::trace!("Trace logging is active.");
}
