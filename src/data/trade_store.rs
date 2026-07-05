use rusqlite::params;

use crate::{
    data::db::{DataError, Db},
    domain::trade::Trade,
};

pub fn insert_trade(db: &Db, trade: &Trade) -> Result<(), DataError> {
    db.connection.execute(
        "insert into trades (event, symbol, quantity, price, executed_at, currency, commission)
         values (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            trade.event.as_str(),
            trade.symbol,
            trade.quantity.to_string(),
            trade.price.to_string(),
            trade.executed_at,
            trade.currency,
            trade.commission.to_string()
        ],
    )?;

    Ok(())
}
