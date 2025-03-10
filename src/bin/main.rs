use trackrs::{ cli, providers };

// #[cfg(not(tarpaulin_include))]
fn main() {
    let provider = providers::JsonProvider::new_today().unwrap();

    let commands = cli::commands();
    let matches = commands.get_matches();

    cli::init_logger(&matches);

    cli::execute(&matches, &provider).expect("error during execution")
}
