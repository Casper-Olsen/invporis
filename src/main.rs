mod cli;
mod data;
mod domain;

use clap::Parser;
use colored::Colorize;

use crate::cli::command::{Commands, RootCommand};
use crate::data::db::{DataError, Db};
use crate::domain::trade::Trade;

fn main() -> Result<(), DataError> {
    let cli = RootCommand::parse();

    match cli.command {
        Commands::Add(args) => {
            let trade = Trade {
                event: args.event,
                symbol: args.symbol,
            };

            println!(
                "Adding trade with event: {}",
                trade.event.to_string().green()
            );

            let db = Db::open()?;
            db.insert_trade(&trade)?;

            Ok(())
        }
        Commands::GetTotalValue => {
            println!("Getting total value ...");
            Ok(())
        }
    }
}
