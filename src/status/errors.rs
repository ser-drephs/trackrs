use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatusError {
    #[error("start not found error")]
    DailyStartError,
}

