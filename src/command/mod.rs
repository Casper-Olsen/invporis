pub mod add;
pub mod get_total_value;
pub mod import;

use crate::{
    apperror::AppError,
    cli::command::{Commands, RootCommand},
    data::{
        db::{Db, DbLocation},
        migration,
    },
};

pub fn execute(root_command: RootCommand, db_location: DbLocation) -> Result<(), AppError> {
    let mut db = Db::open(db_location)?;
    migration::migrate(&mut db)?;

    match root_command.command {
        Commands::Add(args) => add::run(args, &db),
        Commands::Import(args) => import::run(args, &mut db),
        Commands::GetTotalValue => get_total_value::run(),
    }
}
