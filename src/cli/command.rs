use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand, ValueEnum};
use rust_decimal::{Decimal, dec};

#[derive(ValueEnum, Clone, Debug)]
pub enum Event {
    Buy,
    Sell,
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Provider {
    Nordnet,
    Saxo,
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Buy => "buy",
            Self::Sell => "sell",
        };
        write!(f, "{s}")
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct RootCommand {
    /// Commands
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands
#[derive(Subcommand)]
pub enum Commands {
    // Add a trade
    #[command(about = "Add a trade")]
    Add(AddArgs),

    /// Get total value of portfolio
    #[command(about = "Get total value of portfolio")]
    GetTotalValue,

    /// Import trades
    #[command(about = "Import trades")]
    Import(ImportArgs),
}

#[derive(Args)]
pub struct AddArgs {
    #[arg(long, short = 'e', required = true)]
    pub event: Event,

    #[arg(long, short = 'i', required = true)]
    pub isin: String,

    #[arg(long, short = 'q', required = false, default_value_t = dec!(1))]
    pub quantity: Decimal,

    #[arg(long, short = 'p', required = true)]
    pub price: Decimal,

    #[arg(long, required = false, default_value = "DKK")]
    pub price_currency: String,

    #[arg(long, short = 'f', required = false, default_value_t = dec!(0))]
    pub fee: Decimal,

    #[arg(long, required = false, default_value = "DKK")]
    pub fee_currency: String,

    #[arg(long, short = 't', required = true)]
    pub executed_date: NaiveDate,
}

#[derive(Args)]
pub struct ImportArgs {
    #[arg(long, short = 'f', required = true)]
    pub file: PathBuf,

    #[arg(long, short = 'p', required = true)]
    pub provider: Provider,
}
