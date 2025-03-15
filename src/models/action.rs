use std::fmt::{self, Display};

use serde_derive::{ Deserialize, Serialize };

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq, Copy)]
pub enum Action {
    Start,
    Break,
    End,
    Takeover,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::Action;

    #[test]
    fn should_format() {
        let repr = format!("{}", Action::Break);
        assert_eq!(repr, "Break")
    }
}
