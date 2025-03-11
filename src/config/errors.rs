use config::ConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationProviderError {
}

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("building configuration error")]
    BuildError(#[from] ConfigError),
    #[error("configuration provider error")]
    ConfigurationProviderError(#[from] ConfigurationProviderError)
}