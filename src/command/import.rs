use std::{
    fs::File,
    io::{BufRead, BufReader},
};

use crate::{cli::command::ImportArgs, error::AppError};

pub fn import_trades(args: ImportArgs) -> Result<(), AppError> {
    let vec = BufReader::new(File::open(args.path)?)
        .lines()
        .collect::<Result<Vec<_>, _>>()?;

    for elem in &vec {
        println!("{elem}");
    }

    Ok(())
}
