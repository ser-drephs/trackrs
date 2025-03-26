use config::ConfigError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigurationError {
    #[error("config error")] ConfigError(#[from] ConfigError),
    #[error("config file create error")] ConfigFileCreationError(#[from] std::io::Error),
    #[error("config file save error")] ConfigFileSaveError(#[from] serde_json::Error),
}
