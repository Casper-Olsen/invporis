mod cli;
mod command;
mod data;
mod domain;

use clap::Parser;

use crate::{cli::command::RootCommand, data::db::DbLocation};

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    env_logger::init();

    let command = RootCommand::parse();

    if let Err(error) = execute(command, DbLocation::Persisted).await {
        eprintln!("{error:#}");
        std::process::exit(1);
    }
}

async fn execute(root_command: RootCommand, db_location: DbLocation) -> Result<(), anyhow::Error> {
    command::execute(root_command, db_location).await
}

#[cfg(test)]
mod tests {
    use chrono::Local;
    use rust_decimal::dec;

    use crate::{
        cli::command::{AddArgs, Commands, RootCommand},
        data::db::DbLocation,
        execute,
    };

    #[tokio::test]
    async fn test_add_trade() {
        let dkk = String::from("DKK");
        let root_command = RootCommand {
            command: Commands::Add(AddArgs {
                event: crate::cli::command::Event::Buy,
                asset_type: crate::cli::command::AssetType::Security,
                symbol: None,
                isin: Some(String::from("test")),
                quantity: dec!(33),
                price: dec!(100),
                price_currency: dkk.clone(),
                executed_date: Local::now().date_naive(),
                fee_currency: dkk,
                fee: dec!(0),
            }),
        };
        let res = execute(root_command, DbLocation::InMemory).await;
        assert!(res.is_ok());
    }
}
