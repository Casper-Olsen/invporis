use clap::{Args, Parser, Subcommand};

use crate::domain::event::Event;

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
}

#[derive(Args)]
pub struct AddArgs {
    #[arg(long, short, required = true)]
    pub event: Event,

    #[arg(long, short, required = true)]
    pub symbol: String,
}
