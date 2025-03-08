use serde_derive::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) enum Action {
    Start,
    Break,
    End,
    Takeover
}