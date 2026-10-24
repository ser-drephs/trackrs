use std::{fmt::Display, path::Path, process, str::FromStr};

use chrono::{Datelike, IsoWeek, Local, NaiveDate, Utc};
use log::LevelFilter;
use usage::{
    error::UsageErr,
    parse::{ParseOutput, ParseValue},
    Spec,
};

use crate::{
    entry::Status, Settings, StatusDaily, StatusMonthly, StatusWeekly, StatusYearly, Takeover,
    TimeData, TimeDataMonthly, TimeDataWeekly, TimeDataYearly, TrackerError,
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
    Status {
        week: Option<i16>,
        month: Option<i16>,
        year: Option<i16>,
        table: Option<bool>,
    },
    Start {
        note: Option<String>,
    },
    Break {
        note: Option<String>,
    },
    End,
    /// Full day of `sick` or `vacation`, named by the note.
    Absence {
        date: NaiveDate,
        note: String,
    },
    Add {
        minutes: u16,
    },
    Takeover {
        minutes: u16,
    },
    Config {
        list: bool,
        edit: bool,
    },
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
                month: string(&out, "m").map(|m| typed("month", m)).transpose()?,
                year: string(&out, "y").map(|y| typed("year", y)).transpose()?,
                table: opt_flag(&out, "table"),
            },
            "start" => Commands::Start {
                note: string(&out, "note").map(str::to_owned),
            },
            "break" => Commands::Break {
                note: string(&out, "note").map(str::to_owned),
            },
            "end" => Commands::End,
            name @ ("sick" | "vacation") => Commands::Absence {
                date: match string(&out, "date") {
                    Some(d) => typed("date", d)?,
                    None => Utc::now().date_naive(),
                },
                note: name.to_owned(),
            },
            "add" => Commands::Add {
                minutes: typed("minutes", string(&out, "minutes").unwrap_or_default())?,
            },
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
        let verbosity = LevelFilter::iter().nth(level).unwrap_or(LevelFilter::Trace);

        Ok(Cli { command, verbosity })
    }
}

fn value<'a>(out: &'a ParseOutput, name: &str) -> Option<&'a ParseValue> {
    let flag = out
        .flags
        .iter()
        .find(|(f, _)| f.name == name)
        .map(|(_, v)| v);
    flag.or_else(|| {
        out.args
            .iter()
            .find(|(a, _)| a.name == name)
            .map(|(_, v)| v)
    })
}

fn flag(out: &ParseOutput, name: &str) -> bool {
    matches!(value(out, name), Some(ParseValue::Bool(true)))
}

/// `None` when the flag is absent, so the caller can pick its own default.
fn opt_flag(out: &ParseOutput, name: &str) -> Option<bool> {
    match value(out, name) {
        Some(ParseValue::Bool(b)) => Some(*b),
        _ => None,
    }
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
    raw.parse().map_err(|e| {
        (
            format!("error: invalid value '{raw}' for '<{name}>': {e}"),
            2,
        )
    })
}
pub trait CliExecute {
    fn execute(&self) -> TrackerResult;
    fn init_logger(&self) -> TrackerResult;
}

impl CliExecute for Cli {
    fn execute(&self) -> TrackerResult {
        match &self.command {
            Commands::Break { note } => self.invoke_break(note),
            Commands::End => self.invoke_end(),
            Commands::Absence { date, note } => self.invoke_absence(date, note),
            Commands::Status {
                week,
                month,
                year,
                table,
            } => self.invoke_status(week, month, year, table),
            Commands::Config { list: _, edit } => self.invoke_config(edit),
            Commands::Add { minutes } => self.invoke_add(minutes),
            Commands::Takeover { minutes } => self.invoke_takeover(minutes),
            Commands::Start { note } => self.invoke_start(note),
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
    fn invoke_start(&self, note: &Option<String>) -> TrackerResult {
        log::info!("start executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.to_owned().into())
            .today()
            .build()?;
        let now = Local::now();
        time_data.read_from_file()?;
        let first = !time_data
            .entries
            .data
            .iter()
            .any(|e| e.status == Status::Connect);
        time_data.append(Status::Connect, now.to_utc(), note.clone())?;

        // A takeover is added before the first start of a day only, and kept if it doesn't fit.
        let takeover = Takeover::new(Path::new(&settings.folder));
        let mut taken = None;
        if first {
            if let Some(minutes) = takeover.get()? {
                match time_data.prepend(minutes, "takeover") {
                    Ok(_) => taken = Some(minutes),
                    Err(e) => {
                        eprintln!("keeping takeover of {minutes} minutes for the next start: {e}")
                    }
                }
            }
        }

        time_data.write_to_file()?;
        if taken.is_some() {
            takeover.remove()?;
        }
        Ok(())
    }

    fn invoke_break(&self, note: &Option<String>) -> TrackerResult {
        log::info!("break executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        let now = Local::now();
        time_data
            .read_from_file()?
            .append(Status::Break, now.to_utc(), note.clone())?
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
            .append(Status::End, now.to_utc(), None)?
            .assert_break(
                status.exp_break.unwrap().duration,
                status.r#break.unwrap().duration,
            )?
            .write_to_file()?;
        self.invoke_status(&None, &None, &None, &None)
    }

    fn invoke_add(&self, minutes: &u16) -> TrackerResult {
        log::info!("add executed");
        let settings = Settings::new()?;
        let mut time_data = TimeData::builder()
            .folder(settings.folder.into())
            .today()
            .build()?;
        time_data
            .read_from_file()?
            .prepend(*minutes, "manually added")?
            .write_to_file()
    }

    fn invoke_absence(&self, date: &NaiveDate, note: &str) -> TrackerResult {
        log::info!("{note} executed for {date}");
        let settings = Settings::new()?;
        // An empty day starts at 06:00 UTC: a regular workday start across Europe (UTC-1 to UTC+3)
        // that keeps the whole day on the same date there, in UTC and local time.
        let start = date.and_hms_opt(6, 0, 0).unwrap().and_utc();
        let mut time_data = TimeData::builder()
            .folder(settings.folder.to_owned().into())
            .date(start)
            .build()?;
        time_data
            .read_from_file()?
            .absence(note, Local::now().to_utc(), &settings)?
            .write_to_file()
    }

    fn invoke_status(
        &self,
        week: &Option<i16>,
        month: &Option<i16>,
        year: &Option<i16>,
        table: &Option<bool>,
    ) -> TrackerResult {
        log::info!("status executed");
        let settings = Settings::new()?;

        if let Some(y) = year {
            let today = Local::now().date_naive();
            let time_data = TimeDataYearly::load(settings.folder.to_owned().into(), *y, today)?;
            let status = StatusYearly::new(time_data, &settings)?;
            if table.unwrap_or(true) {
                status.format_table();
            } else {
                println!("{}", status);
            }
            return Ok(());
        }

        if let Some(m) = month {
            let today = Local::now().date_naive();
            let time_data = TimeDataMonthly::load(settings.folder.to_owned().into(), *m, today)?;
            let status = StatusMonthly::new(time_data, &settings)?;
            if table.unwrap_or(true) {
                status.format_table();
            } else {
                println!("{}", status);
            }
            return Ok(());
        }

        match week {
            Some(w) => {
                let cur_week: IsoWeek = Local::now().iso_week();
                // Relative weeks count from the current week, which may belong to another year
                // around new year, e.g. 1st of January 2027 is in week 53 of 2026.
                let year = if *w > 0 {
                    Local::now().year()
                } else {
                    cur_week.year()
                };
                let time_data = TimeDataWeekly::builder()
                    .folder(settings.folder.to_owned().into())
                    .year(year.try_into()?)
                    .week(w, cur_week)
                    .build()?;

                let status = StatusWeekly::builder()
                    .data(time_data)
                    .settings(settings)
                    .build()?;

                if table.unwrap_or(true) {
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
                if table.unwrap_or(false) {
                    status.format_ranges_table();
                } else {
                    println!("{}", status);
                }
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
        let takeover = Takeover::new(Path::new(&settings.folder));
        if let Some(pending) = takeover.get()? {
            return Err(TrackerError::TakeoverSetError {
                message: format!(
                    "a takeover of {pending} minutes is still pending for the next start"
                ),
            });
        }

        let mut time_data = TimeData::builder()
            .folder(settings.folder.to_owned().into())
            .today()
            .build()?;
        time_data.read_from_file()?.takeover(*minutes)?;

        // Store the minutes first, so they're never lost; undo that if the day can't be saved.
        takeover.set(*minutes)?;
        if let Err(e) = time_data.write_to_file() {
            takeover.remove()?;
            return Err(e);
        }
        self.invoke_status(&None, &None, &None, &None)
    }
}
