use std::any::Any;

use config::Source;

use super::ConfigurationProviderError;

pub trait ConfigurationProvider {
    fn source(&self) -> Result<Box<dyn Source>, ConfigurationProviderError>;
    fn as_any(&self) -> &dyn Any;
}
