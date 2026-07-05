use rusqlite::params;

use crate::{
    data::db::{DataError, Db},
    domain::trade::Trade,
};

pub fn insert_trade(db: &Db, trade: &Trade) -> Result<(), DataError> {
    db.connection.execute(
        "insert into trades (event, symbol) values (?1, ?2)",
        params![trade.event.as_str(), trade.symbol],
    )?;

    Ok(())
}
