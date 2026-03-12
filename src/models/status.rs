use serde::{ Deserialize, Serialize };

#[derive(Clone, Debug, Serialize, Deserialize, Copy, PartialEq, Eq)]
pub enum Status {
    #[deprecated = "Use Start instead"]
    #[serde(rename(serialize = "Start", deserialize = "Connect"))]
    Connect,
    #[deprecated = "Use Break instead"]
    #[serde(rename(serialize = "Break", deserialize = "Disconnect"))]
    Disconnect,
    Start,
    Break,
    End,
    Takeover,
}
