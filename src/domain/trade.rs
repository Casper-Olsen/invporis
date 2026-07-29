use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::{
    cli,
    command::import::{NordnetEvent, NordnetTrade},
};

#[derive(Clone, Debug)]
pub enum Event {
    Buy,
    Sell,
}

impl Event {
    pub const fn as_str(&self) -> &'static str {
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

#[derive(Debug)]
pub struct Trade {
    pub event: Event,
    pub isin: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub executed_at: DateTime<Utc>,
    pub currency: String,
    pub fee: Decimal,
}

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
            price: nordnet_trade.price,
            executed_at: datetime_utc,
            currency: nordnet_trade.currency,
            fee: nordnet_trade.fee,
        }
    }
}
