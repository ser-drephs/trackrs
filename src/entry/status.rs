use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Status {
    Connect,
    Disconnect,
    Break,
    End,
    Takeover,
}
