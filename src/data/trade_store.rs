use std::collections::HashSet;

use rusqlite::{Transaction, params};

use crate::{
    data::db::Db,
    domain::trade::{Provider, Trade},
    error::AppError,
};

const INSERT_SQL: &str =
    "insert into trades (event, isin, asset_type, symbol, quantity, price, price_currency, fee, fee_currency, executed_date, provider, provider_id)
     values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)";

pub fn insert_trade(db: &Db, trade: &Trade) -> Result<(), AppError> {
    db.connection.execute(
        INSERT_SQL,
        params![
            trade.event.as_str(),
            trade.isin,
            trade.asset_type.as_str(),
            trade.symbol,
            trade.quantity.to_string(),
            trade.price.amount.to_string(),
            trade.price.currency.clone(),
            trade.fee.amount.to_string(),
            trade.fee.currency.clone(),
            trade.executed_date,
            trade.provider.map(crate::domain::trade::Provider::as_str),
            trade.provider_id
        ],
    )?;

    Ok(())
}

pub fn insert_trades(transaction: &Transaction, trades: &Vec<Trade>) -> Result<(), AppError> {
    let mut statement = transaction.prepare(INSERT_SQL)?;

    for trade in trades {
        statement.execute(params![
            trade.event.as_str(),
            trade.isin,
            trade.asset_type.as_str(),
            trade.symbol,
            trade.quantity.to_string(),
            trade.price.amount.to_string(),
            trade.price.currency.clone(),
            trade.fee.amount.to_string(),
            trade.fee.currency.clone(),
            trade.executed_date,
            trade.provider.map(crate::domain::trade::Provider::as_str),
            trade.provider_id
        ])?;
    }

    Ok(())
}

pub fn list_trades(db: &Db, provider: Provider) -> Result<HashSet<String>, AppError> {
    let mut statement = db
        .connection
        .prepare("SELECT provider_id FROM trades WHERE provider = ?1")?;

    let trade_iter =
        statement.query_map(params![provider.as_str()], |row| row.get::<_, String>(0))?;

    trade_iter
        .map(|trade| trade.map_err(AppError::from))
        .collect()
}
