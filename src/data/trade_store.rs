use rusqlite::params;

use crate::{data::db::Db, domain::trade::Trade, error::AppError};

pub fn insert_trade(db: &Db, trade: &Trade) -> Result<(), AppError> {
    db.connection.execute(
        "insert into trades (event, symbol, quantity, price, executed_at, currency, fee)
         values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            trade.event.as_str(),
            trade.symbol,
            trade.quantity.to_string(),
            trade.price.to_string(),
            trade.executed_at,
            trade.currency,
            trade.fee.to_string()
        ],
    )?;

    Ok(())
}
