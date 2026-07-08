mod cli;
mod command;
mod data;
mod domain;
mod error;

use clap::Parser;

use crate::cli::command::RootCommand;
use crate::error::AppError;

fn main() {
    let command = RootCommand::parse();

    if let Err(error) = execute(command) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}

fn execute(root_command: RootCommand) -> Result<(), AppError> {
    command::execute(root_command)
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use rust_decimal::dec;

    use crate::{
        cli::command::{AddArgs, Commands, RootCommand},
        execute,
    };

    #[test]
    fn test_add_trade() {
        let root_command = RootCommand {
            command: Commands::Add(AddArgs {
                event: crate::cli::command::Event::Buy,
                symbol: "test".to_string(),
                quantity: dec!(33),
                price: dec!(100),
                executed_at: Local::now().to_utc(),
                currency: "USD".to_string(),
                fee: dec!(0),
            }),
        };
        let res = execute(root_command);
        println!("{res:?}");
        assert!(res.is_ok());
    }
}
