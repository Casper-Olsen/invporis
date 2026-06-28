mod cli;
mod data;

use clap::Parser;
use colored::Colorize;

use crate::cli::parser::Args;
use crate::data::db::{DataError, Db, Event, Trade};

fn main() -> Result<(), DataError> {
    let args = Args::parse();

    match args {
        Args {
            total_value: true, ..
        } => {
            println!("Getting total value ...");
        }
        Args {
            add_trade: true, ..
        } => {
            println!("Adding trade ...");

            let trade = Trade { event: Event::Buy };

            let db = Db::open()?;
            db.insert_trade(&trade)?;
        }
        _ => {
            eprintln!("{}", "Argument not provided. Exiting...".red());
            return Ok(());
        }
    }

    Ok(())
}
