pub mod add;
pub mod get_total_value;
pub mod import;

use crate::{
    cli::command::{Commands, RootCommand},
    error::AppError,
};

pub fn execute(root_command: RootCommand) -> Result<(), AppError> {
    match root_command.command {
        Commands::Add(args) => add::create_trade(args),
        Commands::Import(args) => import::import_trades(args),
        Commands::GetTotalValue => get_total_value::calculate(),
    }
}
