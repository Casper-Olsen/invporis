use rusqlite::params;

use crate::{data::db::Db, domain::trade::Trade, error::AppError};

const INSERT_SQL: &str =
    "insert into trades (event, isin, quantity, price, executed_at, currency, fee)
             values (?1, ?2, ?3, ?4, ?5, ?6, ?7)";

pub fn insert_trade(db: &Db, trade: &Trade) -> Result<(), AppError> {
    db.connection.execute(
        INSERT_SQL,
        params![
            trade.event.as_str(),
            trade.isin,
            trade.quantity.to_string(),
            trade.price.to_string(),
            trade.executed_at,
            trade.currency,
            trade.fee.to_string()
        ],
    )?;

    Ok(())
}

pub fn insert_trades(db: &mut Db, trades: Vec<Trade>) -> Result<(), AppError> {
    let transaction = db.connection.transaction()?;
    {
        let mut statement = transaction.prepare(INSERT_SQL)?;

        for trade in trades {
            statement.execute(params![
                trade.event.as_str(),
                trade.isin,
                trade.quantity.to_string(),
                trade.price.to_string(),
                trade.executed_at,
                trade.currency,
                trade.fee.to_string()
            ])?;
        }
    }
    transaction.commit()?;

    Ok(())
}
