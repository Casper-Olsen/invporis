use chrono::NaiveDate;
use rust_decimal::Decimal;
use std::hash::Hash;

use crate::{
    cli,
    command::import::{CoinbaseEvent, NordnetEvent, SaxoEvent},
};

#[derive(Debug, Eq, PartialEq, Hash)]
pub enum Trade {
    Equity(EquityTrade),
    Crypto(CryptoTrade),
}

#[derive(Debug)]
pub struct EquityTrade {
    pub event: Event,
    pub isin: String,
    pub symbol: Option<String>,
    pub quantity: Decimal,
    pub price: MonetaryAmount,
    pub fee: MonetaryAmount,
    pub executed_date: NaiveDate,
    pub provider: Option<Provider>,
    pub provider_id: Option<String>,
}

// TODO: Should we have symbol here (and in hash)?
impl PartialEq for EquityTrade {
    fn eq(&self, other: &Self) -> bool {
        self.isin == other.isin && self.price.currency == other.price.currency
    }
}

impl Hash for EquityTrade {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.isin.hash(state);
        self.price.currency.hash(state);
    }
}

impl Eq for EquityTrade {}

#[derive(Debug)]
pub struct CryptoTrade {
    pub event: Event,
    pub symbol: String,
    pub quantity: Decimal,
    pub price: MonetaryAmount,
    pub fee: MonetaryAmount,
    pub executed_date: NaiveDate,
    pub provider: Option<Provider>,
    pub provider_id: Option<String>,
}

impl PartialEq for CryptoTrade {
    fn eq(&self, other: &Self) -> bool {
        self.symbol == other.symbol && self.price.currency == other.price.currency
    }
}

impl Hash for CryptoTrade {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.symbol.hash(state);
        self.price.currency.hash(state);
    }
}

impl Eq for CryptoTrade {}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Deserialize)]
pub enum AssetType {
    #[serde(rename = "equity")]
    Equity,

    #[serde(rename = "crypto")]
    Crypto,
}

impl AssetType {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Equity => "equity",
            Self::Crypto => "crypto",
        }
    }
}

#[derive(Clone, Debug)]
pub struct MonetaryAmount {
    pub amount: Decimal,
    pub currency: String,
}
