use chrono::{DateTime, Utc};
use rust_decimal::Decimal;

use crate::cli;

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

#[derive(Debug)]
pub struct Trade {
    pub event: Event,
    pub symbol: String,
    pub quantity: Decimal,
    pub price: Decimal,
    pub executed_at: DateTime<Utc>,
    pub currency: String,
    pub fee: Decimal,
}
