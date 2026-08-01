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
const SAXO_SHEET_NAME: &str = "Trades";

fn deserialize_comma_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let sanitized = s.replace(',', ".");
    Decimal::from_str(&sanitized).map_err(serde::de::Error::custom)
}

fn deserialize_price<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let Some(index) = s.find(' ') else {
        return Err(serde::de::Error::custom(
            "invalid price format; missing space separator",
        ));
    };

    Decimal::from_str(&s[..index]).map_err(serde::de::Error::custom)
}

fn deserialize_event<'de, D>(deserializer: D) -> Result<SaxoEvent, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let Some(i) = s.find(' ') else {
        return Err(serde::de::Error::custom(
            "invalid event format; missing space separator",
        ));
    };

    let event = &s[..i];
    match event {
        "Buy" => Ok(SaxoEvent::Buy),
        "Sell" => Ok(SaxoEvent::Sell),
        _ => Err(serde::de::Error::custom("invalid event type".to_string())),
    }
}

fn default_saxo_fee_currency() -> String {
    "DKK".to_string()
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

    // TODO: Store fee as negative
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

#[derive(serde::Deserialize)]
pub struct SaxoTrade {
    #[serde(rename = "Event", deserialize_with = "deserialize_event")]
    pub event: SaxoEvent,

    #[serde(rename = "Instrument ISIN")]
    pub isin: String,

    #[serde(rename = "Instrument")]
    pub security_name: String,

    #[serde(rename = "Quantity")]
    pub quantity: Decimal,

    #[serde(rename = "Price", deserialize_with = "deserialize_price")]
    pub price: Decimal,

    #[serde(rename = "Instrument currency")]
    pub price_currency: String,

    #[serde(rename = "Total cost (DKK)")]
    pub fee: Decimal,

    #[serde(default = "default_saxo_fee_currency")]
    pub fee_currency: String,

    #[serde(rename = "Trade Date")]
    pub executed_at: NaiveDate,

    #[serde(rename = "Trade ID")]
    pub id: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum SaxoEvent {
    #[serde(rename = "Buy")]
    Buy,

    #[serde(rename = "Sell")]
    Sell,
}

pub fn run(args: ImportArgs, db: &mut Db) -> Result<(), AppError> {
    match args.provider {
        CliProvider::Nordnet => import_nordnet(args.file, db),
        CliProvider::Saxo => import_saxo(args.file, db),
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
        .flexible(true)
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

// TODO: Implement import
// TODO: Refactor after implementing the whole import and make sure to reuse between Nordnet and Saxo imports
fn import_saxo(file: PathBuf, db: &mut Db) -> Result<(), AppError> {
    ensure_extension(&file, "xlsx")?;

    let mut workbook: Xlsx<_> = open_workbook(file)?;

    if !workbook.sheet_names().contains(&SAXO_SHEET_NAME.to_owned()) {
        return Err(AppError::Import(format!(
            "No {SAXO_SHEET_NAME} sheet in the file"
        )));
    }

    let mut buffer = vec![];
    {
        let mut csv_writer = csv::Writer::from_writer(&mut buffer);

        let range = workbook.worksheet_range(SAXO_SHEET_NAME)?;
        for row in range.rows() {
            let record = row
                .iter()
                .map(data_to_string)
                .collect::<Result<Vec<_>, AppError>>()?;

            csv_writer.write_record(record)?;
        }
        csv_writer.flush()?;
    }

    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .from_reader(buffer.as_slice());

    let trades_to_insert = reader
        .deserialize::<SaxoTrade>()
        .map(|r| r.map(Trade::from).map_err(AppError::from))
        .collect::<Result<Vec<_>, _>>()?;

    trade_store::insert_trades(db, &trades_to_insert)?;

    Ok(())
}

fn data_to_string(data: &calamine::Data) -> Result<String, AppError> {
    match data {
        calamine::Data::Int(i) => Ok(i.to_string()),
        calamine::Data::Float(f) => Ok(f.to_string()),
        calamine::Data::String(s) => Ok(s.clone()),
        calamine::Data::Bool(b) => Ok(b.to_string()),
        calamine::Data::DateTime(d) => {
            let dt = d.as_datetime().ok_or(AppError::Import(
                "Failed to convert Excel datetime to a valid date".to_string(),
            ))?;

            // TODO: What is the timezone here? We need to store in UTC
            // TODO: Should we store with Time here? Right now we are stripping the Time part
            Ok(dt.date().to_string())
        }
        calamine::Data::DateTimeIso(d) | calamine::Data::DurationIso(d) => Ok(d.clone()),
        calamine::Data::Error(e) => Err(AppError::Import(e.to_string())),
        calamine::Data::Empty => Ok(String::new()),
    }
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
