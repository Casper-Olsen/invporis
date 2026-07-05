mod cli;
mod data;

use clap::Parser;
use colored::Colorize;

use crate::cli::command::{Commands, RootCommand};
use crate::data::db::{DataError, Db, Trade};

fn main() {
    if let Err(data_error) = execute() {
        eprintln!("{data_error}");
        std::process::exit(1);
    }
}

fn execute() -> Result<(), DataError> {
    let cli = RootCommand::parse();

    match cli.command {
        Commands::Add(args) => {
            println!(
                "Adding trade with event: {}",
                args.event.to_string().green()
            );

            let trade = Trade {
                event: data::db::Event::from(args.event),
                symbol: args.symbol,
            };

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
