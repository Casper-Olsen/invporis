use chrono::NaiveDate;
use rust_decimal::{Decimal, prelude::Zero};

use crate::{
    cli,
    command::import::{NordnetEvent, NordnetTrade, SaxoEvent, SaxoTrade},
};

#[derive(Debug)]
pub struct Trade {
    pub event: Event,
    pub isin: String,
    pub asset_type: AssetType,
    pub symbol: Option<String>,
    pub quantity: Decimal,
    pub price: MonetaryAmount,
    pub fee: MonetaryAmount,
    pub executed_date: NaiveDate,
    pub provider: Option<Provider>,
    pub provider_id: Option<String>,
}

impl From<NordnetTrade> for Trade {
    fn from(nordnet_trade: NordnetTrade) -> Self {
        Self {
            event: nordnet_trade.event.into(),
            isin: nordnet_trade.isin,
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
            provider: Some(Provider::Nordnet),
            provider_id: Some(nordnet_trade.id),
        }
    }
}

impl From<SaxoTrade> for Trade {
    fn from(saxo_trade: SaxoTrade) -> Self {
        Self {
            event: saxo_trade.event.into(),
            isin: saxo_trade.isin,
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
            provider: Some(Provider::Saxo),
            provider_id: Some(saxo_trade.id),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Event {
    Buy,
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

#[derive(Clone, Copy, Debug)]
pub enum Provider {
    Nordnet,
    Saxo,
}

impl Provider {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Nordnet => "nordnet",
            Self::Saxo => "saxo",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum AssetType {
    Security,
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
