use chrono::NaiveDate;
use rust_decimal::{Decimal, prelude::Zero};

use crate::{
    cli,
    command::import::{
        CoinbaseEvent, CoinbaseTrade, NordnetEvent, NordnetTrade, SaxoEvent, SaxoTrade,
    },
};

#[derive(Debug)]
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
            provider: Some(Provider::Nordnet),
            provider_id: Some(nordnet_trade.id),
        }
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
            provider: Some(Provider::Saxo),
            provider_id: Some(saxo_trade.id),
        }
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
            provider: Some(Provider::Coinbase),
            provider_id: Some(coinbase_trade.id),
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

impl From<CoinbaseEvent> for Event {
    fn from(item: CoinbaseEvent) -> Self {
        match item {
            CoinbaseEvent::Buy => Self::Buy,
            CoinbaseEvent::Sell => Self::Sell,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Provider {
    Nordnet,
    Saxo,
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
