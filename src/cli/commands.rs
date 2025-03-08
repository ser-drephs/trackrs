use clap::Command;


pub fn start_cmd() -> Command{
    return Command::new("start2");
}

/*#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Get the status of current tracking
    ///
    /// Get the status for either a day or a week. Not providing additional options will return status for today.
    #[clap(display_order = 1)]
    Status {
        /// Week to show the status for
        ///
        /// Either enter the correct week of the year or a relative value eg. -1
        #[clap(short, value_parser, allow_hyphen_values = true)]
        week: Option<i8>,

        /// Format week status as table.
        #[clap(short, long)]
        table: bool,
    },
    /// Start tracking work
    ///
    /// Starts tracking work for today.
    #[clap(display_order = 2)]
    Start,
    /// Take a break
    ///
    /// Breaks current tracking.
    #[clap(display_order = 3)]
    Break,
    /// End tracking work
    ///
    /// End tracking work for today.
    #[clap(display_order = 4)]
    End,
    /// Take over time to next day
    ///
    /// Takes over defined minutes to next day, whenever next connect is executed.
    #[clap(display_order = 7)]
    Takeover {
        /// Minutes to take over to next day.
        #[clap()]
        minutes: u16,
    },
    /// Configuration
    ///
    /// List or edit configuration
    #[clap(display_order = 8)]
    Config {
        /// List configuration
        #[clap(short, long, conflicts_with = "edit")]
        list: bool,
        /// Open configuration in default editor
        #[clap(short, long, conflicts_with = "list")]
        edit: bool,
    },
}
*/
