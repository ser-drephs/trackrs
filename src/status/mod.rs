mod daily;
pub use daily::*;

mod daily_data;
pub use daily_data::*;

mod time;
pub use time::*;

mod weekly;
pub use weekly::*;

mod weekly_data;
pub use weekly_data::*;

/// A status that can be shown in different formats. `Display` is the plain text, the other
/// formats have their own method.
pub trait StatusDisplay: std::fmt::Display {
    /// The status as a table.
    fn table(&self) -> String;
}
