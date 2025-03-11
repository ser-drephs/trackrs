mod commands;

use chrono::{Datelike, IsoWeek, Local};
use clap::{Parser, Subcommand};
use log::LevelFilter;

use crate::{
    entry::Status, Settings, StatusDaily, StatusWeekly, TimeData, TimeDataWeekly, TrackerError,
};

type TrackerResult = Result<(), TrackerError>;



impl CliExecute for Cli {
    fn execute(&self) -> TrackerResult {
        match &self.command {
            Commands::Break => self.invoke_break(),
            Commands::End => self.invoke_end(),
            Commands::Disconnect => self.invoke_disconnect(),
            Commands::Statuus { week, table } => self.invoke_status(week, table),
            Commands::Config { list: _, edit } => self.invoke_config(edit),
            Commands::Takeover { minutes } => self.invoke_takeover(minutes),
            Commands::Start => self.invoke_start(),
            _ => self.invoke_continue(), // default and Command::Start.
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
        let now = Local::now();
        time_data
            .read_from_file()?
            .assert_takeover(now)?
            .append(Status::Connect, now)?
            .write_to_file()
    }

    fn invoke_continue(&self) -> TrackerResult {
        log::info!("start executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .append(Status::Connect, now)?
            .write_to_file()
    }

    fn invoke_break(&self) -> TrackerResult {
        log::info!("break executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .append(Status::Break, now)?
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
        let now = Local::now();
        time_data
            .append(Status::End, now)?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration,
            )?
            .write_to_file()?;
        self.invoke_status(&None, &false)
    }

    fn invoke_disconnect(&self) -> TrackerResult {
        log::info!("disconnect executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .append(Status::Disconnect, now)?
            .write_to_file()
    }

    fn invoke_status(&self, week: &Option<i8>, table: &bool) -> TrackerResult {
        log::info!("status executed");
        let settings = Settings::new()?;

        match week {
            Some(w) => {
                let year = Local::now().year();
                let cur_week: IsoWeek = Local::now().iso_week();
                let time_data = TimeDataWeekly::builder()
                    .folder(settings.folder.to_owned().into())
                    .year(year.try_into()?)
                    .week(w, cur_week)
                    .build()?;

                let status = StatusWeekly::builder()
                    .data(time_data)
                    .settings(settings)
                    .build()?;

                if *table {
                    status.format_table();
                } else {
                    println!("{}", status);
                }
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
        } else {
            println!("{:#?}", settings);
        }
        Ok(())
    }

    fn invoke_takeover(&self, minutes: &u16) -> TrackerResult {
        log::info!("takeover {} minutes", minutes);
        let settings = Settings::new()?;
        let folder: &str = settings.folder.as_ref();
        let mut time_data = TimeData::builder().folder(folder.into()).today().build()?;
        time_data.read_from_file()?;
        let status = StatusDaily::builder()
            .data(time_data.clone())
            .settings(settings)
            .build()?;
        let now = Local::now();
        time_data
            .append(Status::End, now)?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration,
            )?
            .write_to_file()?;
        self.invoke_status(&None, &false)
    }
}
