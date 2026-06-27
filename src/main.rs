mod cli;
mod data;

use clap::Parser;

use crate::cli::parser::Args;
use crate::data::sqlite::{self, DataError};

fn main() -> Result<(), DataError> {
    let args = Args::parse();

    if args.total_value {
        println!("Getting total value ...");
    } else {
        eprintln!("Exiting!");
        return Ok(());
    }

    sqlite::initialize()?;

    Ok(())
}
