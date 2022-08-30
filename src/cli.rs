use chrono::{Datelike, Local};
use clap::{Parser, Subcommand};
use log::LevelFilter;

use crate::{
    entry::Status, Settings, StatusDaily, StatusWeekly, TimeData, TimeDataWeekly, TrackerError,
};

type TrackerResult = Result<(), TrackerError>;

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

#[derive(Subcommand, Debug)]
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
        week: Option<i8>
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
    /// Disconnect from work
    ///
    /// Simple disconnect from work. This will only create a disconnect entry in the tracking history.
    #[clap(display_order = 5)]
    Disconnect,
    /// Continue tracking work
    ///
    /// Continue tracking work for today.
    #[clap(display_order = 6)]
    Continue,
    /// Configuration
    ///
    /// List or edit configuration
    #[clap(display_order = 7)]
    Config {
        /// List configuration
        #[clap(short, long, conflicts_with="edit")]
        list: bool,
        /// Open configuration in default editor
        #[clap(short, long, conflicts_with="list")]
        edit: bool,
    },
}

pub trait CliExecute {
    fn execute(&self) -> TrackerResult;
    fn init_logger(&self) -> TrackerResult;
}

impl CliExecute for Cli {
    fn execute(&self) -> TrackerResult {
        match &self.command {
            Commands::Break => self.invoke_break(),
            Commands::End => self.invoke_end(),
            Commands::Disconnect => self.invoke_disconnect(),
            Commands::Status { week } => self.invoke_status(week),
            Commands::Config { list: _, edit } => self.invoke_config(edit),
            _ => self.invoke_start(), // default and Command::Start.
        }
    }

    fn init_logger(&self) -> TrackerResult {
        let verbosity = self.verbose.log_level_filter();

        let mut builder = env_logger::builder();
        let logger = builder.filter_level(verbosity).format_target(false);

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

        Ok(())
    }
}

impl Cli {
    fn invoke_start(&self) -> TrackerResult {
        log::info!("start executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        time_data
            .read_from_file()?
            .append(Status::Connect)?
            .write_to_file()
    }

    fn invoke_break(&self) -> TrackerResult {
        log::info!("break executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        time_data
            .read_from_file()?
            .append(Status::Break)?
            .write_to_file()
    }

    fn invoke_end(&self) -> TrackerResult {
        log::info!("end executed");
        let settings = Settings::new()?;
        let folder: &str = settings.folder.as_ref();
        let mut time_data = TimeData::builder().folder(folder.into()).today().build()?;
        time_data.read_from_file()?;
        let status = StatusDaily::builder()
            .data(time_data.clone())
            .settings(settings)
            .build()?;
        time_data
            .append(Status::End)?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration
            )?
            .write_to_file()?;
        self.invoke_status(&None)
    }

    fn invoke_disconnect(&self) -> TrackerResult {
        log::info!("disconnect executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        time_data
            .read_from_file()?
            .append(Status::Disconnect)?
            .write_to_file()
    }

    fn invoke_status(&self, week: &Option<i8>) -> TrackerResult {
        log::info!("status executed");
        let settings = Settings::new()?;

        match week {
            Some(w) => {
                let year = Local::now().year();
                let time_data = TimeDataWeekly::builder()
                    .folder(settings.folder.to_owned().into())
                    .year(year.try_into()?)
                    .week(w, None)
                    .build()?;

                let status = StatusWeekly::builder()
                    .data(time_data)
                    .settings(settings)
                    .build()?;
                println!("{}", status);
            }
            None => {
                let mut time_data = TimeData::builder()
                    .folder(settings.folder.to_owned().into())
                    .today()
                    .build()?;
                time_data.read_from_file()?;
                let status = StatusDaily::builder()
                    .data(time_data)
                    .settings(settings)
                    .build()?;
                println!("{}", status);
            }
        }
        Ok(())
    }

    fn invoke_config(&self, edit: &bool) -> TrackerResult {
        log::info!("status executed");
        let settings = Settings::new()?;

        if *edit {
            log::debug!("invoke default editor with config");
            open::that(settings.file)?
        } else{
            println!("{:#?}", settings);
        }
        Ok(())
    }
}
