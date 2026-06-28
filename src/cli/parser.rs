use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, long_about = None)]
pub struct Args {
    /// Get total value of portfolio
    #[arg(short = 't', long = "total-value")]
    pub total_value: bool,
}
