use rusqlite::{Transaction, params};
use std::collections::HashSet;

use crate::domain::security::Security;
use crate::{data::db::Db, error::AppError};

pub fn insert_securities(
    transaction: &Transaction,
    securities: &HashSet<Security>,
) -> Result<(), AppError> {
    let mut statement = transaction.prepare(
        "insert into securities (isin, currency, name)
             values (?1, ?2, ?3)",
    )?;

    for security in securities {
        statement.execute(params![security.isin, security.currency, security.name])?;
    }

    Ok(())
}

pub fn list_keys(db: &Db) -> Result<HashSet<(String, String)>, AppError> {
    let mut statement = db
        .connection
        .prepare("SELECT isin, currency FROM securities")?;

    let trade_iter = statement.query_map([], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
    })?;

    trade_iter
        .map(|trade| trade.map_err(AppError::from))
        .collect()
}
