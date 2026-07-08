use crate::{cli::command::ImportArgs, error::AppError};

pub fn import_trades(args: ImportArgs) -> Result<(), AppError> {
    let path_exists = args.path.try_exists();
    Ok(())
}
