use calamine::{Reader, Xlsx, open_workbook};
use csv::ReaderBuilder;
use encoding_rs::UTF_16LE;
use log::debug;
use std::path::{Path, PathBuf};

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
    ensure_extension(&file, "csv")?;

    let bytes = std::fs::read(file)?;
    let (content, encoding_used, has_errors) = UTF_16LE.decode(&bytes);

    if has_errors {
        return Err(AppError::Import(
            "File contains invalid UTF-16LE text".to_string(),
        ));
    }

    debug!("Decoded {} bytes", bytes.len());
    debug!("Decoded using {}", encoding_used.name());

    let mut reader = ReaderBuilder::new()
        .delimiter(b';')
        .from_reader(content.as_bytes());

    for record in reader.records() {
        println!("{:?}", record.unwrap());
    }

    Ok(())
}

fn import_saxo(file: PathBuf) -> Result<(), AppError> {
    ensure_extension(&file, "xlsx")?;

    let mut workbook: Xlsx<_> = open_workbook(file)?;

    let names = workbook.sheet_names();
    let Some(sheet_name) = names.first() else {
        return Err(AppError::Import("No sheet in the file".to_string()));
    };

    debug!("Using \"{sheet_name}\" sheet");

    if let Ok(range) = workbook.worksheet_range(sheet_name) {
        for l in range.rows() {
            println!("{l:?}");
        }
    }

    Ok(())
}

fn ensure_extension(file: &Path, expected_extension: &str) -> Result<(), AppError> {
    let Some(extension) = file.extension() else {
        return Err(AppError::Import("File has no extension".to_string()));
    };

    if !extension.eq_ignore_ascii_case(expected_extension) {
        return Err(AppError::Import("File has incorrect extension".to_string()));
    }

    Ok(())
}
