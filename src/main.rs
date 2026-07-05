mod cli;
mod data;
mod domain;

use clap::Parser;
use colored::Colorize;

use crate::cli::command::{Commands, RootCommand};
use crate::data::db::{DataError, Db};
use crate::data::trade_store;
use crate::domain::trade::Trade;

fn main() {
    let command = RootCommand::parse();

    if let Err(data_error) = execute(command) {
        eprintln!("{data_error}");
        std::process::exit(1);
    }
}

fn execute(root_command: RootCommand) -> Result<(), DataError> {
    match root_command.command {
        Commands::Add(args) => {
            println!(
                "Adding trade with event: {}",
                args.event.to_string().green()
            );

            let trade = Trade {
                event: domain::trade::Event::from(args.event),
                symbol: args.symbol,
                quantity: args.quantity,
                price: args.price,
                executed_at: args.executed_at,
                currency: args.currency,
                commission: args.commision,
            };

            let db = Db::open()?;
            trade_store::insert_trade(&db, &trade)?;

            Ok(())
        }
        Commands::GetTotalValue => {
            println!("Getting total value ...");
            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use rust_decimal::dec;

    use crate::{
        cli::command::{AddArgs, Commands, RootCommand},
        execute,
    };

    #[test]
    fn test_add_trade() {
        let root_command = RootCommand {
            command: Commands::Add(AddArgs {
                event: crate::cli::command::Event::Buy,
                symbol: "test".to_string(),
                quantity: dec!(33),
                price: dec!(100),
                executed_at: Local::now().to_utc(),
                currency: "USD".to_string(),
                commision: dec!(0),
            }),
        };
        let res = execute(root_command);
        assert!(res.is_ok());
    }
}
