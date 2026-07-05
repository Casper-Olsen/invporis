use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(ValueEnum, Clone, Debug)]
pub enum Event {
    Buy,
}

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Buy => "buy",
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
}

#[derive(Args)]
pub struct AddArgs {
    #[arg(long, short, required = true)]
    pub event: Event,

    #[arg(long, short, required = true)]
    pub symbol: String,
}
