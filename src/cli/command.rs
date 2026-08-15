use std::path::PathBuf;

use chrono::NaiveDate;
use clap::{Args, Parser, Subcommand, ValueEnum};
use rust_decimal::{Decimal, dec};

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

    /// Get value of portfolio (defaults to the total value of the portfolio)
    #[command(about = "Get value of portfolio")]
    GetValue,

    /// Import trades
    #[command(about = "Import trades from file")]
    Import(ImportArgs),
}

#[derive(Args)]
pub struct AddArgs {
    #[arg(long, short = 'e', required = false, default_value_t = Event::Buy)]
    pub event: Event,

    #[arg(long, short = 'a', required = false, default_value_t = AssetType::Security)]
    pub asset_type: AssetType,

    #[arg(long, short = 's', required = false)]
    pub symbol: Option<String>,

    #[arg(long, short = 'i', required = false)]
    pub isin: Option<String>,

    #[arg(long, short = 'q', required = false, default_value_t = dec!(1))]
    pub quantity: Decimal,

    #[arg(long, short = 'p', required = true)]
    pub price: Decimal,

    #[arg(long, required = false, default_value = "DKK")]
    pub price_currency: String,

    #[arg(long, short = 'd', required = true)]
    pub executed_date: NaiveDate,

    #[arg(long, short = 'f', required = false, default_value_t = dec!(0))]
    pub fee: Decimal,

    #[arg(long, required = false, default_value = "DKK")]
    pub fee_currency: String,
}

#[derive(Args)]
pub struct ImportArgs {
    #[arg(long, short = 'f', required = true)]
    pub file: PathBuf,

    #[arg(long, short = 'p', required = true)]
    pub provider: Provider,
}
#[derive(ValueEnum, Clone, Debug)]
pub enum Event {
    Buy,
    Sell,
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

#[derive(ValueEnum, Clone, Debug)]
pub enum AssetType {
    Security,
    Crypto,
}

impl std::fmt::Display for AssetType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Security => "security",
            Self::Crypto => "crypto",
        };
        write!(f, "{s}")
    }
}

#[derive(ValueEnum, Clone, Debug)]
pub enum Provider {
    Nordnet,
    Saxo,
    Coinbase,
}
