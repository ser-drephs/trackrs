use std::{fmt::Display, process, str::FromStr};

use chrono::{Datelike, IsoWeek, Local};
use log::LevelFilter;
use usage::{
    error::UsageErr,
    parse::{ParseOutput, ParseValue},
    Spec,
};

use crate::{
    entry::Status, Settings, StatusDaily, StatusWeekly, TimeData, TimeDataWeekly, TrackerError,
};

type TrackerResult = Result<(), TrackerError>;

/// Usage spec describing the commands and options of the CLI.
const SPEC: &str = include_str!("cli.usage.kdl");

/// Simple time tracker using CLI.
///
/// A simple time tracker using the CLI. Writes an entry with the current timestamp for each command that is invoked.
#[derive(Debug)]
pub struct Cli {
    command: Commands,
    verbosity: LevelFilter,
}

#[derive(Debug)]
pub enum Commands {
    Status { week: Option<i8>, table: bool },
    Start,
    Break,
    End,
    Takeover { minutes: u16 },
    Config { list: bool, edit: bool },
}

/// Message to print and exit code to terminate with, when parsing does not yield a command.
type ParseExit = (String, i32);

impl Cli {
    /// Parse the process arguments, exiting on help, version or invalid input.
    pub fn parse() -> Self {
        Self::parse_from(std::env::args())
    }

    /// Parse the given arguments, exiting on help, version or invalid input.
    pub fn parse_from<I, T>(args: I) -> Self
    where
        I: IntoIterator<Item = T>,
        T: ToString,
    {
        let args: Vec<String> = args.into_iter().map(|a| a.to_string()).collect();
        Self::try_parse(&args).unwrap_or_else(|(msg, code)| {
            if code == 0 {
                println!("{msg}");
            } else {
                eprintln!("{msg}");
            }
            process::exit(code)
        })
    }

    fn try_parse(args: &[String]) -> Result<Self, ParseExit> {
        let mut spec: Spec = SPEC.parse().expect("cli usage spec is valid");
        spec.version = Some(env!("CARGO_PKG_VERSION").to_owned());

        let out = usage::Parser::new(&spec)
            .explain(args)
            .map_err(|e| (e.to_string(), 2))?;
        if let Some(e) = out
            .errors
            .iter()
            .find(|e| matches!(e, UsageErr::Help(_) | UsageErr::Version(_)))
        {
            return Err((e.to_string(), 0));
        }
        if !out.errors.is_empty() {
            let msg = out.errors.iter().map(ToString::to_string);
            return Err((msg.collect::<Vec<_>>().join("\n"), 2));
        }

        let command = match out.cmd.name.as_str() {
            "status" => Commands::Status {
                week: string(&out, "w").map(|w| typed("week", w)).transpose()?,
                table: flag(&out, "table"),
            },
            "start" => Commands::Start,
            "break" => Commands::Break,
            "end" => Commands::End,
            "takeover" => Commands::Takeover {
                minutes: typed("minutes", string(&out, "minutes").unwrap_or_default())?,
            },
            "config" => Commands::Config {
                list: flag(&out, "list"),
                edit: flag(&out, "edit"),
            },
            other => return Err((format!("unrecognized subcommand '{other}'"), 2)),
        };

        // Same mapping as clap-verbosity-flag: errors by default, -v/-q step up/down.
        let level = (1 + count(&out, "verbose")).saturating_sub(count(&out, "quiet"));
        let verbosity = LevelFilter::iter()
            .nth(level)
            .unwrap_or(LevelFilter::Trace);

        Ok(Cli { command, verbosity })
    }
}

fn value<'a>(out: &'a ParseOutput, name: &str) -> Option<&'a ParseValue> {
    let flag = out.flags.iter().find(|(f, _)| f.name == name).map(|(_, v)| v);
    flag.or_else(|| out.args.iter().find(|(a, _)| a.name == name).map(|(_, v)| v))
}

fn flag(out: &ParseOutput, name: &str) -> bool {
    matches!(value(out, name), Some(ParseValue::Bool(true)))
}

fn count(out: &ParseOutput, name: &str) -> usize {
    match value(out, name) {
        Some(ParseValue::MultiBool(v)) => v.iter().filter(|b| **b).count(),
        _ => 0,
    }
}

fn string<'a>(out: &'a ParseOutput, name: &str) -> Option<&'a str> {
    match value(out, name) {
        Some(ParseValue::String(s)) => Some(s),
        _ => None,
    }
}

fn typed<T>(name: &str, raw: &str) -> Result<T, ParseExit>
where
    T: FromStr,
    T::Err: Display,
{
    raw.parse()
        .map_err(|e| (format!("error: invalid value '{raw}' for '<{name}>': {e}"), 2))
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
            Commands::Status { week, table } => self.invoke_status(week, table),
            Commands::Config { list: _, edit } => self.invoke_config(edit),
            Commands::Takeover { minutes } => self.invoke_takeover(minutes),
            Commands::Start => self.invoke_start(),
        }
    }

    fn init_logger(&self) -> TrackerResult {
        let verbosity = self.verbosity;

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
            .assert_takeover(now.to_utc())?
            .append(Status::Connect, now.to_utc())?
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
            .append(Status::Break, now.to_utc())?
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
            .append(Status::End, now.to_utc())?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration,
            )?
            .write_to_file()?;
        self.invoke_status(&None, &false)
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
            .append(Status::End, now.to_utc())?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration,
            )?
            .write_to_file()?;
        self.invoke_status(&None, &false)
    }
}
