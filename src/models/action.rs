use serde_derive::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum Action {
    Start,
    Break,
    End,
    Takeover
}