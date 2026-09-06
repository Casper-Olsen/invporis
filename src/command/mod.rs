pub mod add;
pub mod get_value;
pub mod import;

use crate::{
    cli::command::{Commands, RootCommand},
    data::{
        db::{Db, DbLocation},
        migration,
    },
};

pub async fn execute(
    root_command: RootCommand,
    db_location: DbLocation,
) -> Result<(), anyhow::Error> {
    let mut db = Db::open(db_location)?;
    migration::migrate(&mut db)?;

    match root_command.command {
        Commands::AddEquity(args) => add::run_for_equity(args, &db),
        Commands::AddCrypto(args) => add::run_for_crypto(args, &db),
        Commands::Import(args) => import::run(args, db),
        Commands::GetValue => get_value::run(db).await,
    }
}
