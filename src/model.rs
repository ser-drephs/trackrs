use serde::{Serialize, Deserialize};

/// Status of the data entry.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Connect,
    Disconnect,
    Break,
    End,
    Takeover
}
