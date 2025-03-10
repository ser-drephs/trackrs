use config::ConfigError;
use thiserror::Error;

use crate::providers;

#[derive(Error, Debug)]
pub enum TrackerError {
    #[error("dafuq")]
    Unknown,
    // #[error("parse error")]
    // ParseError(#[from] serde_json::Error),
    #[error("conversion error")]
    ConverstionError(#[from] std::convert::Infallible),
    #[error("io error")]
    IoError(#[from] std::io::Error),
    #[error("configuration error")]
    TrackerConfigError(#[from] ConfigError),
    #[error("week number conversion")]
    TrackerIntError(#[from] std::num::TryFromIntError),
    #[error("takeover not parsable")]
    TrackerTakeoverParseError(#[from] std::num::ParseIntError),
    #[error("entry data not set: {message}")]
    EntryError { message: String },
    #[error("time data error: {message}")]
    TimeDataError { message: String },
    #[error("week crosses year: {message}")]
    TimeDataWeekCrossesYear { message: String },
    #[error("status error: {message}")]
    StatusError { message: String },
    #[error("weekly status error: {message}")]
    StatusWeeklyError { message: String },
    #[error("set takeover error: {message}")]
    TakeoverSetError { message: String },
    #[error("get takeover error: {message}")]
    TakeoverGetError { message: String },
    // interal error conversion
    #[error("provider error")]
    ProviderError(#[from] providers::ProviderError),
}
