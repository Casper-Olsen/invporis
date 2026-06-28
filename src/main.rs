mod cli;
mod data;

use clap::Parser;
use colored::Colorize;

use crate::cli::parser::Args;
use crate::data::db::{DataError, Db};

fn main() -> Result<(), DataError> {
    let args = Args::parse();

    if args.total_value {
        println!("Getting total value ...");
    } else {
        eprintln!("{}", "Argument not provided. Exiting...".red());
        return Ok(());
    }

    let db = Db::open()?;

    Ok(())
}
