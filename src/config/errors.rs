use config::ConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("building configuration error")] BuildError(#[from] ConfigError),
    #[error("create configuration file error")] CreationError(#[from] std::io::Error),
    #[error("save configuration error")] SaveError(#[from] serde_json::Error),
}
