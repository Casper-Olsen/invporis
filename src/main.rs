mod cli;
mod command;
mod data;
mod domain;
mod error;

use clap::Parser;

use crate::cli::command::RootCommand;
use crate::data::db::DbLocation;
use crate::error::AppError;

fn main() {
    env_logger::init();

    let command = RootCommand::parse();

    if let Err(error) = execute(command, DbLocation::Persisted) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn execute(root_command: RootCommand, db_location: DbLocation) -> Result<(), AppError> {
    command::execute(root_command, db_location)
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use rust_decimal::dec;

    use crate::{
        cli::command::{AddArgs, Commands, RootCommand},
        data::db::DbLocation,
        execute,
    };

    #[test]
    fn test_add_trade() {
        let dkk = "DKK".to_string();
        let root_command = RootCommand {
            command: Commands::Add(AddArgs {
                event: crate::cli::command::Event::Buy,
                isin: "test".to_string(),
                quantity: dec!(33),
                price: dec!(100),
                price_currency: dkk.clone(),
                fee_currency: dkk,
                fee: dec!(0),
                executed_date: Local::now().to_utc(),
            }),
        };
        let res = execute(root_command, DbLocation::InMemory);
        println!("{res:?}");
        assert!(res.is_ok());
    }
}
