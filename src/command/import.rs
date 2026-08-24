use anyhow::anyhow;
use calamine::{Reader, Xlsx, open_workbook};
use chrono::{NaiveDate, NaiveDateTime};
use csv::{ReaderBuilder, StringRecord};
use encoding_rs::UTF_16LE;
use log::debug;
use rust_decimal::{Decimal, prelude::Zero as _};
use serde::{Deserialize, de::DeserializeOwned};
use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    str::FromStr,
};

use crate::{
    cli::command::{ImportArgs, Provider as CliProvider},
    data::{db::Db, security_store, trade_store},
    domain::{
        security::Security,
        trade::{AssetType, MonetaryAmount, Provider as DomainProvider, Trade},
    },
};

pub fn run(args: ImportArgs, mut db: Db) -> Result<(), anyhow::Error> {
    match args.provider {
        CliProvider::Nordnet => import_nordnet(args.file, &mut db),
        CliProvider::Saxo => import_saxo(args.file, &mut db),
        CliProvider::Coinbase => import_coinbase(args.file, &mut db),
    }
}

trait ImportTrade: DeserializeOwned {
    fn to_security(&self) -> Option<Security>;
    fn into_trade(self) -> Trade;
}

fn import<T>(
    db: &mut Db,
    provider: DomainProvider,
    records: impl Iterator<Item = Result<StringRecord, csv::Error>>,
    headers: &StringRecord,
) -> Result<(), anyhow::Error>
where
    T: ImportTrade,
{
    let trades = trade_store::list_trade_ids(db, provider)?;
    let mut existing_securities = security_store::list_keys(db)?;

    let mut securities_to_insert = HashSet::<Security>::new();
    let mut trades_to_insert = Vec::new();

    let mut parsed = 0;
    let mut skipped = 0;
    for result in records {
        let record = result?;

        // We allow deserialization errors because CSV files from different providers
        // may contain a mix of trades, transfers, and other data. This allows valid
        // trades to be imported without failing the entire import.
        let source_trade = match record.deserialize::<T>(Some(headers)) {
            Ok(trade) => trade,
            Err(err) => {
                debug!("Failed to deserialize record: {err}");
                skipped += 1;
                continue;
            }
        };

        let security = source_trade.to_security();
        let trade = source_trade.into_trade();

        parsed += 1;

        // We don't want to insert the same trade multiple times
        if trade
            .provider_id
            .as_ref()
            .is_none_or(|id| trades.contains(id))
        {
            continue;
        }

        if let Some(security) = security
            && existing_securities.insert(security.key())
        {
            securities_to_insert.insert(security);
        }

        trades_to_insert.push(trade);
    }

    let transaction = db.connection.transaction()?;

    security_store::insert_securities(&transaction, &securities_to_insert)?;
    trade_store::insert_trades(&transaction, &trades_to_insert)?;

    transaction.commit()?;

    debug!(
        "Parsed {} trades, imported {} new trades",
        parsed,
        trades_to_insert.len(),
    );
    if skipped > 0 {
        debug!("Skipped {skipped} records that could not be deserialized as trades");
    }
    debug!("Imported {} new securities", securities_to_insert.len());

    Ok(())
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
    pub executed_date: NaiveDate,

    #[serde(rename = "Id")]
    pub id: String,
}

impl ImportTrade for NordnetTrade {
    fn to_security(&self) -> Option<Security> {
        Some(Security {
            isin: self.isin.clone(),
            name: Some(self.security_name.clone()),
            currency: self.price_currency.clone(),
        })
    }

    fn into_trade(self) -> Trade {
        self.into()
    }
}

impl From<NordnetTrade> for Trade {
    fn from(nordnet_trade: NordnetTrade) -> Self {
        Self {
            event: nordnet_trade.event.into(),
            isin: Some(nordnet_trade.isin),
            asset_type: AssetType::Security,
            symbol: None,
            quantity: nordnet_trade.quantity,
            price: MonetaryAmount {
                amount: nordnet_trade.price,
                currency: nordnet_trade.price_currency,
            },
            fee: MonetaryAmount {
                amount: if nordnet_trade.fee.is_zero() {
                    Decimal::zero()
                } else {
                    // Nordnet reports fees as positive amounts.
                    -nordnet_trade.fee.abs()
                },
                currency: nordnet_trade.fee_currency,
            },
            executed_date: nordnet_trade.executed_date,
            provider: Some(DomainProvider::Nordnet),
            provider_id: Some(nordnet_trade.id),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum NordnetEvent {
    #[serde(rename = "KØBT")]
    Buy,

    #[serde(rename = "SOLGT")]
    Sell,
}

fn import_nordnet(file: PathBuf, db: &mut Db) -> Result<(), anyhow::Error> {
    ensure_extension(&file, "csv")?;

    let bytes = std::fs::read(file)?;
    let (content, encoding_used, has_errors) = UTF_16LE.decode(&bytes);

    if has_errors {
        return Err(anyhow!("File contains invalid UTF-16LE text"));
    }

    debug!("Decoded {} bytes", bytes.len());
    debug!("Decoded using {}", encoding_used.name());

    let mut reader = ReaderBuilder::new()
        .delimiter(b'\t')
        .flexible(true)
        .from_reader(content.as_bytes());

    let csv_headers = reader.headers()?.clone();
    let headers = into_nordnet_headers(&csv_headers);

    import::<NordnetTrade>(db, DomainProvider::Nordnet, reader.records(), &headers)?;

    Ok(())
}

fn into_nordnet_headers(csv_headers: &StringRecord) -> StringRecord {
    const VALUTA: &str = "valuta";

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

fn default_saxo_fee_currency() -> String {
    String::from("DKK")
}

#[derive(serde::Deserialize)]
pub struct SaxoTrade {
    #[serde(rename = "Event", deserialize_with = "deserialize_saxo_event")]
    pub event: SaxoEvent,

    #[serde(rename = "Instrument ISIN")]
    pub isin: String,

    #[serde(rename = "Instrument Symbol")]
    pub symbol: String,

    #[serde(rename = "Instrument")]
    pub security_name: String,

    #[serde(rename = "Quantity")]
    pub quantity: Decimal,

    #[serde(rename = "Price", deserialize_with = "deserialize_saxo_price")]
    pub price: Decimal,

    #[serde(rename = "Instrument currency")]
    pub price_currency: String,

    #[serde(rename = "Total cost (DKK)")]
    pub fee: Decimal,

    #[serde(default = "default_saxo_fee_currency")]
    pub fee_currency: String,

    #[serde(rename = "Trade Date")]
    pub executed_date: NaiveDate,

    #[serde(rename = "Trade ID")]
    pub id: String,
}

impl ImportTrade for SaxoTrade {
    fn to_security(&self) -> Option<Security> {
        Some(Security {
            isin: self.isin.clone(),
            name: Some(self.security_name.clone()),
            currency: self.price_currency.clone(),
        })
    }

    fn into_trade(self) -> Trade {
        self.into()
    }
}

impl From<SaxoTrade> for Trade {
    fn from(saxo_trade: SaxoTrade) -> Self {
        Self {
            event: saxo_trade.event.into(),
            isin: Some(saxo_trade.isin),
            asset_type: AssetType::Security,
            symbol: Some(saxo_trade.symbol),
            quantity: saxo_trade.quantity,
            price: MonetaryAmount {
                amount: saxo_trade.price,
                currency: saxo_trade.price_currency,
            },
            fee: MonetaryAmount {
                amount: saxo_trade.fee,
                currency: saxo_trade.fee_currency,
            },
            executed_date: saxo_trade.executed_date,
            provider: Some(DomainProvider::Saxo),
            provider_id: Some(saxo_trade.id),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum SaxoEvent {
    #[serde(rename = "Buy")]
    Buy,

    #[serde(rename = "Sell")]
    Sell,
}

fn import_saxo(file: PathBuf, db: &mut Db) -> Result<(), anyhow::Error> {
    const SAXO_SHEET_NAME: &str = "Trades";

    ensure_extension(&file, "xlsx")?;

    let mut workbook: Xlsx<_> = open_workbook(file)?;

    if !workbook.sheet_names().contains(&SAXO_SHEET_NAME.to_owned()) {
        return Err(anyhow!("No {SAXO_SHEET_NAME} sheet in the file"));
    }

    let mut buffer = vec![];
    {
        let mut csv_writer = csv::Writer::from_writer(&mut buffer);

        let range = workbook.worksheet_range(SAXO_SHEET_NAME)?;
        for row in range.rows() {
            let record = row
                .iter()
                .map(data_to_string)
                .collect::<Result<Vec<_>, anyhow::Error>>()?;

            csv_writer.write_record(record)?;
        }
        csv_writer.flush()?;
    }

    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .from_reader(buffer.as_slice());

    let headers = reader.headers()?.clone();
    import::<SaxoTrade>(db, DomainProvider::Saxo, reader.records(), &headers)?;

    Ok(())
}

fn data_to_string(data: &calamine::Data) -> Result<String, anyhow::Error> {
    match data {
        calamine::Data::Int(i) => Ok(i.to_string()),
        calamine::Data::Float(f) => Ok(f.to_string()),
        calamine::Data::String(s) => Ok(s.clone()),
        calamine::Data::Bool(b) => Ok(b.to_string()),
        calamine::Data::DateTime(d) => {
            let dt = d
                .as_datetime()
                .ok_or_else(|| anyhow!("Failed to convert Excel datetime to a valid date"))?;

            Ok(dt.date().to_string())
        }
        calamine::Data::DateTimeIso(d) | calamine::Data::DurationIso(d) => Ok(d.clone()),
        calamine::Data::Error(e) => Err(anyhow!(e.to_string())),
        calamine::Data::Empty => Ok(String::new()),
    }
}

fn deserialize_saxo_event<'de, D>(deserializer: D) -> Result<SaxoEvent, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    let Some(i) = s.find(' ') else {
        return Err(serde::de::Error::custom(
            "invalid event format; missing space separator",
        ));
    };

    // Expected format like: "Buy 72 @ 166.80 DKK"
    let event = &s[..i];
    match event {
        "Buy" => Ok(SaxoEvent::Buy),
        "Sell" => Ok(SaxoEvent::Sell),
        _ => Err(serde::de::Error::custom(String::from("invalid event type"))),
    }
}

fn deserialize_saxo_price<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Expected format like: "166.8 DKK"
    let Some(index) = s.find(' ') else {
        return Err(serde::de::Error::custom(
            "invalid price format; missing space separator",
        ));
    };

    Decimal::from_str(&s[..index]).map_err(serde::de::Error::custom)
}

#[derive(Clone, Debug, serde::Deserialize)]
pub enum CoinbaseEvent {
    #[serde(rename = "Buy")]
    Buy,

    #[serde(rename = "Sell")]
    Sell,
}

#[derive(serde::Deserialize)]
pub struct CoinbaseTrade {
    #[serde(rename = "Transaction Type")]
    pub event: CoinbaseEvent,

    #[serde(rename = "Asset")]
    pub symbol: String,

    #[serde(rename = "Quantity Transacted")]
    pub quantity: Decimal,

    #[serde(
        rename = "Price at Transaction",
        deserialize_with = "deserialize_coinbase_price"
    )]
    pub price: Decimal,

    #[serde(rename = "Price Currency")]
    pub price_currency: String,

    #[serde(rename = "Fees and/or Spread")]
    pub fee: Decimal,

    #[serde(
        rename = "Timestamp",
        deserialize_with = "deserialize_coinbase_timestamp"
    )]
    pub executed_date: NaiveDate,

    #[serde(rename = "ID")]
    pub id: String,
}

impl ImportTrade for CoinbaseTrade {
    fn to_security(&self) -> Option<Security> {
        None
    }

    fn into_trade(self) -> Trade {
        self.into()
    }
}

impl From<CoinbaseTrade> for Trade {
    fn from(coinbase_trade: CoinbaseTrade) -> Self {
        Self {
            event: coinbase_trade.event.into(),
            isin: None,
            asset_type: AssetType::Crypto,
            symbol: Some(coinbase_trade.symbol),
            quantity: coinbase_trade.quantity,
            price: MonetaryAmount {
                amount: coinbase_trade.price,
                currency: coinbase_trade.price_currency.clone(),
            },
            fee: MonetaryAmount {
                amount: if coinbase_trade.fee.is_zero() {
                    Decimal::zero()
                } else {
                    // Coinbase reports fees as positive amounts.
                    -coinbase_trade.fee.abs()
                },
                currency: coinbase_trade.price_currency,
            },
            executed_date: coinbase_trade.executed_date,
            provider: Some(DomainProvider::Coinbase),
            provider_id: Some(coinbase_trade.id),
        }
    }
}

fn import_coinbase(file: PathBuf, db: &mut Db) -> Result<(), anyhow::Error> {
    ensure_extension(&file, "csv")?;

    let mut reader = ReaderBuilder::new()
        .flexible(true)
        .has_headers(false)
        .from_path(file)?;

    // The CSV contains metadata rows before the actual transaction header.
    // Skip those rows until we reach the header row identified by the "ID" column.
    let mut records = reader
        .records()
        .skip_while(|res| res.as_ref().is_ok_and(|sr| sr.get(0) != Some("ID")));

    // Convert to Result<Option<_>> so `?` can propagate CSV errors while preserving a missing header as None.
    let headers = records.next().transpose()?;

    let Some(headers) = headers else {
        return Err(anyhow!(
            "Could not find the header row (expected a row starting with 'ID')",
        ));
    };

    import::<CoinbaseTrade>(db, DomainProvider::Coinbase, records, &headers)?;

    Ok(())
}

fn deserialize_coinbase_price<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Expected format like: "$43086.79"
    s.find(|c: char| c.is_numeric()).map_or_else(
        || Err(serde::de::Error::custom("No price found")),
        |pos| Decimal::from_str(&s[pos..]).map_err(serde::de::Error::custom),
    )
}

fn deserialize_coinbase_timestamp<'de, D>(deserializer: D) -> Result<NaiveDate, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    // Expected format like: "2022-01-16 09:52:22 UTC"
    let naive = NaiveDateTime::parse_from_str(s.as_str(), "%Y-%m-%d %H:%M:%S %Z")
        .map_err(serde::de::Error::custom)?;

    Ok(naive.date())
}

fn deserialize_comma_decimal<'de, D>(deserializer: D) -> Result<Decimal, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    let sanitized = s.replace(',', ".");
    Decimal::from_str(&sanitized).map_err(serde::de::Error::custom)
}

fn ensure_extension(file: &Path, expected_extension: &str) -> Result<(), anyhow::Error> {
    let Some(extension) = file.extension() else {
        return Err(anyhow!("File has no extension"));
    };

    if !extension.eq_ignore_ascii_case(expected_extension) {
        return Err(anyhow!("File has incorrect extension"));
    }

    Ok(())
}
