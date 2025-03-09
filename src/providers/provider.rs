use crate::models::Entries;

use super::ProviderError;

pub trait Provider{
    fn read(&self) -> Result<Entries, ProviderError>;
    fn write(&self, entries: &Entries) -> Result<(), ProviderError>;
}