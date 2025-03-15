use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatusError {
    #[error("format error")] FormatError(#[from] std::fmt::Error),
    #[error("could not compare '{duration}' to 0")] ZeroComparisonError {
        duration: String,
    },
}
