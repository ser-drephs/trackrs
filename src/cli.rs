use chrono::{Datelike, IsoWeek, Local};
use clap::{Parser, Subcommand};
use log::LevelFilter;

use crate::config::Configuration;
use crate::storage_provider::StorageProvider;
use crate::{models::Status, StatusDaily, StatusWeekly, TimeData, TimeDataWeekly, TrackerError};

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

pub trait CliExecute {
    fn execute<P: StorageProvider>(
        &self,
        storage_provider: &P,
        configuration: &Configuration,
    ) -> TrackerResult;
    fn init_logger(&self) -> TrackerResult;
}

impl CliExecute for Cli {
    fn execute<P: StorageProvider>(
        &self,
        storage_provider: &P,
        configuration: &Configuration,
    ) -> TrackerResult {
        match &self.command {
            Commands::Break => self.invoke_break(configuration),
            Commands::End => self.invoke_end(configuration),
            Commands::Disconnect => self.invoke_disconnect(configuration),
            Commands::Status { week, table } => self.invoke_status(week, table, configuration),
            Commands::Config { list: _, edit } => self.invoke_config(edit, configuration),
            Commands::Takeover { minutes } => self.invoke_takeover(minutes, configuration),
            Commands::Start => self.invoke_start(configuration),
            _ => self.invoke_continue(configuration), // default and Command::Start.
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
    fn invoke_start(&self, configuration: &Configuration) -> TrackerResult {
        log::info!("start executed");
        let mut time_data = TimeData::builder()
            .folder((*configuration.folder).into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .assert_takeover(now.to_utc())?
            .append(Status::Connect, now.to_utc())?
            .write_to_file()
    }

    fn invoke_continue(&self, configuration: &Configuration) -> TrackerResult {
        log::info!("start executed");
        let mut time_data = TimeData::builder()
            .folder((*configuration.folder).into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .append(Status::Connect, now.to_utc())?
            .write_to_file()
    }

    fn invoke_break(&self, configuration: &Configuration) -> TrackerResult {
        log::info!("break executed");
        let mut time_data = TimeData::builder()
            .folder((*configuration.folder).into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .append(Status::Break, now.to_utc())?
            .write_to_file()
    }

    fn invoke_end(&self, configuration: &Configuration) -> TrackerResult {
        log::info!("end executed");
        let folder: &str = configuration.folder.as_ref();
        let mut time_data = TimeData::builder().folder(folder.into()).today().build()?;
        time_data.read_from_file()?;
        let status = StatusDaily::builder()
            .data(time_data.clone())
            .settings(configuration.clone())
            .build()?;
        let now = Local::now();
        time_data
            .append(Status::End, now.to_utc())?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration,
            )?
            .write_to_file()?;
        self.invoke_status(&None, &false, configuration)
    }

    fn invoke_disconnect(&self, configuration: &Configuration) -> TrackerResult {
        log::info!("disconnect executed");
        let mut time_data = TimeData::builder()
            .folder((*configuration.folder).into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .append(Status::Disconnect, now.to_utc())?
            .write_to_file()
    }

    fn invoke_status(
        &self,
        week: &Option<i8>,
        table: &bool,
        configuration: &Configuration,
    ) -> TrackerResult {
        log::info!("status executed");
        match week {
            Some(w) => {
                let year = Local::now().year();
                let cur_week: IsoWeek = Local::now().iso_week();
                let time_data = TimeDataWeekly::builder()
                    .folder(configuration.folder.to_owned().into())
                    .year(year.try_into()?)
                    .week(w, cur_week)
                    .build()?;

                let status = StatusWeekly::builder()
                    .data(time_data)
                    .settings(configuration.clone())
                    .build()?;

                if *table {
                    status.format_table();
                } else {
                    println!("{}", status);
                }
            }
            None => {
                let mut time_data = TimeData::builder()
                    .folder(configuration.folder.to_owned().into())
                    .today()
                    .build()?;
                time_data.read_from_file()?;
                let status = StatusDaily::builder()
                    .data(time_data)
                    .settings(configuration.clone())
                    .build()?;
                println!("{}", status);
            }
        }
        Ok(())
    }

    fn invoke_config(&self, edit: &bool, configuration: &Configuration) -> TrackerResult {
        log::info!("status executed");

        if *edit {
            log::debug!("invoke default editor with config");
            open::that(&Configuration::file())?
        } else {
            println!("{:#?}", configuration);
        }
        Ok(())
    }

    fn invoke_takeover(&self, minutes: &u16, configuration: &Configuration) -> TrackerResult {
        log::info!("takeover {} minutes", minutes);
        let folder: &str = configuration.folder.as_ref();
        let mut time_data = TimeData::builder().folder(folder.into()).today().build()?;
        time_data.read_from_file()?;
        let status = StatusDaily::builder()
            .data(time_data.clone())
            .settings(configuration.clone())
            .build()?;
        let now = Local::now();
        time_data
            .append(Status::End, now.to_utc())?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration,
            )?
            .write_to_file()?;
        self.invoke_status(&None, &false, configuration)
    }
}
