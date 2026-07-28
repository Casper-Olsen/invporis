use csv::ReaderBuilder;
use encoding_rs::UTF_16LE;
use log::debug;

use crate::{cli::command::ImportArgs, error::AppError};

pub fn import_trades(args: ImportArgs) -> Result<(), AppError> {
    let bytes = std::fs::read(args.path)?;
    let (content, encoding_used, has_errors) = UTF_16LE.decode(&bytes);

    if has_errors {
        return Err(AppError::ImportError("Error importing"));
    }

    debug!("decoded {} bytes", bytes.len());
    debug!("decoded using {}", encoding_used.name());

    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(content.as_bytes());

    for record in reader.records() {
        println!("{:?}", record.unwrap());
    }

    Ok(())
}
