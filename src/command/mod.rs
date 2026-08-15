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
        Commands::Add(args) => add::run(args, &db),
        Commands::Import(args) => import::run(args, db),
        Commands::GetValue => get_value::run(db).await,
    }
}
