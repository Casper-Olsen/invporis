use chrono::NaiveDate;
use rust_decimal::Decimal;

use crate::{
    cli,
    command::import::{CoinbaseEvent, NordnetEvent, SaxoEvent},
};

pub struct Trade {
    pub event: Event,
    pub isin: Option<String>,
    pub asset_type: AssetType,
    pub symbol: Option<String>,
    pub quantity: Decimal,
    pub price: MonetaryAmount,
    pub fee: MonetaryAmount,
    pub executed_date: NaiveDate,
    pub provider: Option<Provider>,
    pub provider_id: Option<String>,
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum Event {
    #[serde(rename = "buy")]
    Buy,

    #[serde(rename = "sell")]
    Sell,
}

impl Event {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        }
    }
}

impl From<cli::command::Event> for Event {
    fn from(item: cli::command::Event) -> Self {
        match item {
            cli::command::Event::Buy => Self::Buy,
            cli::command::Event::Sell => Self::Sell,
        }
    }
}

impl From<NordnetEvent> for Event {
    fn from(item: NordnetEvent) -> Self {
        match item {
            NordnetEvent::Buy => Self::Buy,
            NordnetEvent::Sell => Self::Sell,
        }
    }
}

impl From<SaxoEvent> for Event {
    fn from(item: SaxoEvent) -> Self {
        match item {
            SaxoEvent::Buy => Self::Buy,
            SaxoEvent::Sell => Self::Sell,
        }
    }
}

impl From<CoinbaseEvent> for Event {
    fn from(item: CoinbaseEvent) -> Self {
        match item {
            CoinbaseEvent::Buy => Self::Buy,
            CoinbaseEvent::Sell => Self::Sell,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Deserialize)]
pub enum Provider {
    #[serde(rename = "nordnet")]
    Nordnet,

    #[serde(rename = "saxo")]
    Saxo,

    #[serde(rename = "coinbase")]
    Coinbase,
}

impl Provider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nordnet => "nordnet",
            Self::Saxo => "saxo",
            Self::Coinbase => "coinbase",
        }
    }
}

#[derive(serde::Deserialize, Clone, Copy, Debug)]
pub enum AssetType {
    #[serde(rename = "security")]
    Security,

    #[serde(rename = "crypto")]
    Crypto,
}

impl From<crate::cli::command::AssetType> for AssetType {
    fn from(value: crate::cli::command::AssetType) -> Self {
        match value {
            cli::command::AssetType::Security => Self::Security,
            cli::command::AssetType::Crypto => Self::Crypto,
        }
    }
}

impl AssetType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Security => "security",
            Self::Crypto => "crypto",
        }
    }
}

#[derive(Clone, Debug)]
pub struct MonetaryAmount {
    pub amount: Decimal,
    pub currency: String,
}
