mod daily;
pub use daily::*;

mod time;
pub use time::*;

/// A status that can be shown in different formats. `Display` is the plain text, the other
/// formats have their own method.
pub trait StatusDisplay: std::fmt::Display {
    /// The status as a table.
    fn table(&self) -> String;
}
