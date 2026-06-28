use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Get total value of portfolio
    #[arg(short = 't', long = "total-value")]
    pub total_value: bool,

    /// Add trade
    #[arg(short = 'a', long = "add-trade")]
    pub add_trade: bool,
}
