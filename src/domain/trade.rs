use std::hash::Hash;

use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::{
    cli,
    command::import::{NordnetEvent, NordnetTrade},
};

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

#[derive(Clone, Copy, Debug)]
pub enum Provider {
    Nordnet,
    #[allow(dead_code)]
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

#[derive(Clone, Debug)]
pub struct MonetaryAmount {
    pub amount: Decimal,
    pub currency: String,
}

#[derive(Debug)]
pub struct Trade {
    pub event: Event,
    pub isin: String,
    pub quantity: Decimal,
    pub price: MonetaryAmount,
    pub fee: MonetaryAmount,
    pub executed_at: DateTime<Utc>,
    pub provider: Option<Provider>,
    pub provider_id: Option<String>,
}

#[derive(Debug)]
pub struct Security {
    pub isin: String,
    pub name: Option<String>,
    pub currency: String,
}

impl PartialEq for Security {
    fn eq(&self, other: &Self) -> bool {
        self.isin == other.isin && self.currency == other.currency
    }
}

impl Hash for Security {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.isin.hash(state);
        self.currency.hash(state);
    }
}

impl Eq for Security {}

impl From<NordnetTrade> for Trade {
    fn from(nordnet_trade: NordnetTrade) -> Self {
        let datetime_utc = nordnet_trade
            .executed_at
            .and_hms_opt(0, 0, 0)
            .expect("00:00:00 is always a valid time")
            .and_utc();

        Self {
            event: nordnet_trade.event.into(),
            isin: nordnet_trade.isin,
            quantity: nordnet_trade.quantity,
            price: MonetaryAmount {
                amount: nordnet_trade.price,
                currency: nordnet_trade.price_currency,
            },
            fee: MonetaryAmount {
                amount: nordnet_trade.fee,
                currency: nordnet_trade.fee_currency,
            },
            executed_at: datetime_utc,
            provider: Some(Provider::Nordnet),
            provider_id: Some(nordnet_trade.id),
        }
    }
}
