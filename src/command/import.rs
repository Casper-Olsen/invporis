use calamine::{Reader, Xlsx, open_workbook};
use chrono::NaiveDate;
use csv::{ReaderBuilder, StringRecord};
use encoding_rs::UTF_16LE;
use log::debug;
use rust_decimal::Decimal;
use serde::Deserialize;
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    cli::command::{ImportArgs, Provider as CliProvider},
    data::{db::Db, security_store, trade_store},
    domain::trade::{Provider as DomainProvider, Security, Trade},
    error::AppError,
};

const VALUTA: &str = "valuta";

fn deserialize_comma_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let sanitized = s.replace(',', ".");
    Decimal::from_str(&sanitized).map_err(serde::de::Error::custom)
}

#[derive(serde::Deserialize)]
pub struct NordnetTrade {
    #[serde(rename = "Transaktionstype")]
    pub event: NordnetEvent,

    #[serde(rename = "ISIN")]
    pub isin: String,

    #[serde(rename = "Værdipapirer")]
    pub security_name: String,

    #[serde(
        rename = "Totalt antal",
        deserialize_with = "deserialize_comma_decimal"
    )]
    pub quantity: Decimal,

    #[serde(rename = "Kurs", deserialize_with = "deserialize_comma_decimal")]
    pub price: Decimal,

    #[serde(rename = "Indkøbsværdi valuta")]
    pub price_currency: String,

    #[serde(
        rename = "Samlede afgifter",
        deserialize_with = "deserialize_comma_decimal"
    )]
    pub fee: Decimal,

    #[serde(rename = "Samlede afgifter valuta")]
    pub fee_currency: String,

    #[serde(rename = "Handelsdag")]
    pub executed_at: NaiveDate,

    #[serde(rename = "Id")]
    pub id: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum NordnetEvent {
    #[serde(rename = "KØBT")]
    Buy,

    #[serde(rename = "SOLGT")]
    Sell,
}

pub fn run(args: ImportArgs, db: &mut Db) -> Result<(), AppError> {
    match args.provider {
        CliProvider::Nordnet => import_nordnet(args.file, db),
        CliProvider::Saxo => import_saxo(args.file),
    }
}

fn import_nordnet(file: PathBuf, db: &mut Db) -> Result<(), AppError> {
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
        .delimiter(b'\t')
        .has_headers(true)
        .flexible(true)
        .quote(b'"')
        .from_reader(content.as_bytes());

    let csv_headers = reader.headers()?.clone();
    let headers = into_nordnet_headers(&csv_headers);

    let nordnet_trades = trade_store::list_trades(db, DomainProvider::Nordnet)?;
    let existing_securities = security_store::list_keys(db)?;

    let mut securities_to_insert = HashSet::<Security>::new();
    let mut trades_to_insert = Vec::new();

    let mut processed = 0;
    for result in reader.records() {
        let record = result?;
        let trade = record.deserialize::<NordnetTrade>(Some(&headers))?;

        processed += 1;

        if nordnet_trades.contains(&trade.id) {
            continue;
        }

        let security_key = (trade.isin.clone(), trade.price_currency.clone());

        if !existing_securities.contains(&security_key) {
            securities_to_insert.insert(Security {
                isin: trade.isin.clone(),
                currency: trade.price_currency.clone(),
                name: Some(trade.security_name.clone()),
            });
        }

        trades_to_insert.push(Trade::from(trade));
    }

    security_store::insert_securities(db, &securities_to_insert)?;
    trade_store::insert_trades(db, &trades_to_insert)?;

    debug!(
        "Processed {} trades, imported {} new trades",
        processed,
        trades_to_insert.len()
    );
    debug!("Imported {} new securities", securities_to_insert.len());

    Ok(())
}

fn into_nordnet_headers(csv_headers: &StringRecord) -> StringRecord {
    let mut headers = StringRecord::new();

    // Nordnet exports currency as a separate "Valuta" column immediately following
    // the associated value column. Rename these pairs to "<field> valuta" so each
    // currency column has a unique header, and skip the standalone "Valuta" headers.
    let mut peekable = csv_headers.iter().peekable();
    while let Some(header) = peekable.next() {
        if header.eq_ignore_ascii_case(VALUTA) {
            continue;
        }

        headers.push_field(header);

        if let Some(next_header) = peekable.peek()
            && next_header.eq_ignore_ascii_case(VALUTA)
        {
            headers.push_field(format!("{header} {VALUTA}").as_str());
        }
    }
    headers
}

fn import_saxo(file: PathBuf) -> Result<(), AppError> {
    ensure_extension(&file, "xlsx")?;

    let mut workbook: Xlsx<_> = open_workbook(file)?;

    // TODO: Is the sheet name with buy/sell always called "Shares"?
    let names = workbook.sheet_names();
    let Some(sheet_name) = names.first() else {
        return Err(AppError::Import("No sheet in the file".to_string()));
    };

    debug!("Using \"{sheet_name}\" sheet");

    // TODO: Implement Saxo import
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
