use clap::{ arg, command, value_parser, Arg, ArgAction, ArgMatches, Command };
use log::LevelFilter;

use crate::{ models::{ Action, TrackerError }, providers::Provider, timesheet };

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

pub fn execute<P: Provider>(matches: &ArgMatches, provider: &P) -> Result<(), TrackerError> {
    match matches.subcommand() {
        Some((START_CMD, _)) => timesheet::Timesheet::add(provider, Action::Start),
        Some((BREAK_CMD, _)) => timesheet::Timesheet::add(provider, Action::Break),
        _ => unreachable!("Exhausted list of subcommands and subcommand_required prevents `None`"),
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

    use crate::cli::commands::status_command;

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
}

/*#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get the status of current tracking
    ///
    /// Get the status for either a day or a week. Not providing additional options will return status for today.
    #[clap(display_order = 1)]
    Status {
        /// Week to show the status for
        ///
        /// Either enter the correct week of the year or a relative value eg. -1
        #[clap(short, value_parser, allow_hyphen_values = true)]
        week: Option<i8>,

        /// Format week status as table.
        #[clap(short, long)]
        table: bool,
    },
    /// Start tracking work
    ///
    /// Starts tracking work for today.
    #[clap(display_order = 2)]
    Start,
    /// Take a break
    ///
    /// Breaks current tracking.
    #[clap(display_order = 3)]
    Break,
    /// End tracking work
    ///
    /// End tracking work for today.
    #[clap(display_order = 4)]
    End,
    /// Take over time to next day
    ///
    /// Takes over defined minutes to next day, whenever next connect is executed.
    #[clap(display_order = 7)]
    Takeover {
        /// Minutes to take over to next day.
        #[clap()]
        minutes: u16,
    },
    /// Configuration
    ///
    /// List or edit configuration
    #[clap(display_order = 8)]
    Config {
        /// List configuration
        #[clap(short, long, conflicts_with = "edit")]
        list: bool,
        /// Open configuration in default editor
        #[clap(short, long, conflicts_with = "list")]
        edit: bool,
    },
}
*/
