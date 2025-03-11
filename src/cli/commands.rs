use clap::{ arg, command, value_parser, Arg, ArgAction, ArgMatches, Command };
use log::LevelFilter;

use crate::{
    config::ConfigurationProvider,
    models::{ Action, TrackerError },
    storage::StorageProvider,
    Tracker,
};

const START_CMD: &str = "start";
const BREAK_CMD: &str = "break";
const END_CMD: &str = "end";
const STATUS_CMD: &str = "status";
const TAKEOVER_CMD: &str = "takeover";
const CONFIG_CMD: &str = "config";

pub fn commands() -> Command {
    command!()
        .subcommand_required(true)
        .disable_help_subcommand(true)
        .arg(
            Arg::new("verbose")
                .short('v')
                .help("Verbose logging")
                .action(ArgAction::Count)
                .global(true)
        )
        .subcommand(Command::new(START_CMD).about("Start tracking work"))
        .subcommand(Command::new(BREAK_CMD).about("Break tracking work"))
        .subcommand(Command::new(END_CMD).about("End tracking work"))
        .subcommand(status_command())
        .subcommand(takeover_command())
        .subcommand(config_command())
}

pub fn init_logger(matches: &ArgMatches) {
    let v = matches.get_count("verbose");

    let verbosity = match v {
        0 => log::Level::Warn,
        1 => log::Level::Info,
        2 => log::Level::Debug,
        _ => log::Level::Trace,
    };

    let mut builder = env_logger::builder();
    let logger = builder.filter_level(verbosity.to_level_filter()).format_target(false);

    if verbosity >= LevelFilter::Debug {
        logger.format_target(true);
    }

    let err = logger.try_init();
    if err.is_err() {
        eprintln!("{:?}", err.unwrap_err());
    }

    log::info!("Logging info.");
    log::debug!("logging debug.");
    log::trace!("Logging trace.");
}

pub fn execute<P: StorageProvider, C: ConfigurationProvider>(
    matches: &ArgMatches,
    storage: &P,
    config: &C
) -> Result<(), TrackerError> {
    match matches.subcommand() {
        Some((START_CMD, _)) => Tracker::add(storage, Action::Start),
        Some((BREAK_CMD, _)) => Tracker::add(storage, Action::Break),
        Some((END_CMD, _)) => Tracker::add(storage, Action::End),
        Some((STATUS_CMD, _)) => {
            if matches.contains_id("week") {
                Tracker::status_week(
                    storage,
                    config,
                    matches.get_one::<u8>("week").unwrap(),
                    matches.get_flag("table")
                )
            } else {
                Tracker::status(storage, config)
            }
        }
        Some((TAKEOVER_CMD, _)) => todo!(),
        Some((CONFIG_CMD, _)) => todo!(),
        _ => unreachable!("Exhausted list of subcommands"),
    }
}

fn status_command() -> Command {
    Command::new(STATUS_CMD)
        .about("Get the status of current tracking")
        .long_about(
            "Get the status for either a day or a week. Not providing additional options will return status for today."
        )
        .arg(
            arg!(week: -w --week "Week to show the status for")
                .action(ArgAction::Set)
                .required(false)
                .long_help("Either enter the correct week of the year or a relative value eg. -1")
                .value_parser(value_parser!(i8).range(-51..51))
                .allow_negative_numbers(true)
        )
        .arg(arg!(-t --table "Format week status as table.").requires("week"))
}

fn config_command() -> Command {
    Command::new(CONFIG_CMD)
        .about("Tracking configuration")
        .long_about("List or edit configuration.")
        .arg(arg!(List: -l --list "List configuration").conflicts_with("Edit"))
        .arg(arg!(Edit: -e --edit "Edit configuration").conflicts_with("List"))
}

fn takeover_command() -> Command {
    Command::new(TAKEOVER_CMD)
        .hide(true)
        .about("Take over time to next day")
        .long_about("Takes over defined minutes to next day, whenever next connect is executed.")
        .arg(
            Arg::new("minutes")
                .action(ArgAction::Set)
                .index(1)
                .help("Minutes to take over to next day.")
                .required(true)
                .value_parser(value_parser!(u16).range(1..720))
        )
}

#[cfg(test)]
mod tests {
    use clap::error::ErrorKind;

    use crate::cli::commands::{ self, status_command, takeover_command };

    use super::config_command;

    #[test]
    fn should_conflict_config_edit_and_list() {
        assert!(config_command().try_get_matches_from(vec!["config", "-e", "-l"]).is_err());
        let res = config_command().try_get_matches_from(vec!["config", "-l", "-e"]);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::ArgumentConflict)
    }

    #[test]
    fn should_allow_plain_status() {
        let res = status_command().try_get_matches_from(vec!["status"]);
        assert!(res.is_ok());
        let matches = res.as_ref().unwrap();
        assert_eq!(matches.get_one::<i8>("week"), None);
        assert!(!matches.get_flag("table"))
    }

    #[test]
    fn should_parse_status_week() {
        let res = status_command().try_get_matches_from(vec!["status", "-w", "2"]);
        assert!(res.is_ok());
        match res.as_ref().unwrap().get_one::<i8>("week") {
            Some(number) => assert_eq!(number, &2),
            None => assert!(false),
        }
    }

    #[test]
    fn should_parse_status_week_negative_number() {
        let res = status_command().try_get_matches_from(vec!["status", "-w", "-2"]);
        assert!(res.is_ok());
        match res.as_ref().unwrap().get_one::<i8>("week") {
            Some(number) => assert_eq!(number, &-2),
            None => assert!(false),
        }
    }

    #[test]
    fn should_not_parse_status_week_out_of_range() {
        let res = status_command().try_get_matches_from(vec!["status", "-w", "72"]);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn should_not_parse_status_week_out_of_range_negative_value() {
        let res = status_command().try_get_matches_from(vec!["status", "-w", "-72"]);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::ValueValidation);
    }

    #[test]
    fn should_fail_status_table_without_week() {
        let res = status_command().try_get_matches_from(vec!["status", "-t"]);
        assert_eq!(res.unwrap_err().kind(), ErrorKind::MissingRequiredArgument)
    }

    #[test]
    fn should_allow_status_table_week() {
        let res = status_command().try_get_matches_from(vec!["status", "-w", "3", "-t"]);
        assert!(res.is_ok());
        assert!(res.unwrap().get_flag("table"))
    }

    #[test]
    fn should_require_takeover_minutes() {
        let res = takeover_command().try_get_matches_from(vec!["takeover"]);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::MissingRequiredArgument)
    }

    #[test]
    fn should_accepts_takeover_minutes_first_arg() {
        let res = takeover_command().try_get_matches_from(vec!["takeover", "12"]);
        assert!(res.is_ok());
        let binding = res.unwrap();
        let minutes = binding.get_one::<u16>("minutes").expect("could not parse takeover");
        assert_eq!(minutes, &12);
    }

    #[test]
    fn should_panic_takeover_minutes_explicit() {
        let res = takeover_command().try_get_matches_from(vec!["takeover", "-m", "18"]);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::UnknownArgument)
    }

    #[test]
    fn should_panic_unknown_subcommand() {
        let res = commands::commands().try_get_matches_from(vec!["trackrs", "something"]);
        assert!(res.is_err());
        assert_eq!(res.unwrap_err().kind(), ErrorKind::InvalidSubcommand)
    }

    #[test]
    fn should_parse_multiple_v_flags() {
        let res = commands::commands().try_get_matches_from(vec!["trackrs", "-vvv", "start"]);
        assert!(res.is_ok());
        let verbose = res.unwrap().get_count("verbose");
        assert_eq!(verbose, 3)
    }

    #[test]
    fn should_parse_multiple_v_flags_last() {
        let res = commands::commands().try_get_matches_from(vec!["trackrs", "start", "-vv"]);
        assert!(res.is_ok());
        let verbose = res.unwrap().get_count("verbose");
        assert_eq!(verbose, 2)
    }
}
