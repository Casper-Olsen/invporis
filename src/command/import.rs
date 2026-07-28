use std::path::PathBuf;

use calamine::{Reader, Xlsx, open_workbook};
use csv::ReaderBuilder;
use encoding_rs::UTF_16LE;
use log::debug;

use crate::{
    cli::command::{ImportArgs, Provider},
    error::AppError,
};

pub fn import_trades(args: ImportArgs) -> Result<(), AppError> {
    match args.provider {
        Provider::Nordnet => import_nordnet(args.file),
        Provider::Saxo => import_saxo(args.file),
    }
}

fn import_nordnet(file: PathBuf) -> Result<(), AppError> {
    let bytes = std::fs::read(file)?;
    let (content, encoding_used, has_errors) = UTF_16LE.decode(&bytes);

    if has_errors {
        return Err(AppError::Import(
            "File contains invalid UTF-16LE text".to_string(),
        ));
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

fn import_saxo(file: PathBuf) -> Result<(), AppError> {
    let mut workbook: Xlsx<_> = open_workbook(file)?;

    if let Ok(range) = workbook.worksheet_range("Trades") {
        for l in range.rows() {
            println!("{l:?}");
        }
    }

    Ok(())
}
