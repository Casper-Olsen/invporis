use chrono::NaiveDate;
use rusqlite::{Transaction, params};
use rust_decimal::Decimal;
use serde_rusqlite::from_rows;
use std::collections::HashSet;

use crate::{
    data::db::Db,
    domain::trade::{MonetaryAmount, Provider, Trade},
};

const INSERT_SQL: &str =
    "insert into trades (event, isin, asset_type, symbol, quantity, price, price_currency, fee, fee_currency, executed_date, provider, provider_id)
     values (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)";

const SELECT_SQL: &str = "SELECT event, isin, asset_type, symbol, quantity, price, price_currency, fee, fee_currency, executed_date, provider, provider_id FROM trades";

pub fn insert_trade(db: &Db, trade: &Trade) -> Result<(), anyhow::Error> {
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

pub fn insert_trades(transaction: &Transaction, trades: &Vec<Trade>) -> Result<(), anyhow::Error> {
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

pub fn list_trade_ids(db: &Db, provider: Provider) -> Result<HashSet<String>, anyhow::Error> {
    let mut statement = db
        .connection
        .prepare("SELECT provider_id FROM trades WHERE provider = ?1")?;

    let trade_iter =
        statement.query_map(params![provider.as_str()], |row| row.get::<_, String>(0))?;

    trade_iter
        .map(|trade| trade.map_err(anyhow::Error::from))
        .collect()
}

#[derive(serde::Deserialize)]
struct TradeRow {
    pub event: crate::domain::trade::Event,
    pub isin: Option<String>,
    pub asset_type: crate::domain::trade::AssetType,
    pub symbol: Option<String>,
    pub quantity: Decimal,
    pub price: Decimal,
    pub price_currency: String,
    pub fee: Decimal,
    pub fee_currency: String,
    pub executed_date: NaiveDate,
    pub provider: Option<Provider>,
    pub provider_id: Option<String>,
}

impl From<TradeRow> for Trade {
    fn from(trade_row: TradeRow) -> Self {
        Self {
            event: trade_row.event,
            isin: trade_row.isin,
            asset_type: trade_row.asset_type,
            symbol: trade_row.symbol,
            quantity: trade_row.quantity,
            price: MonetaryAmount {
                amount: trade_row.price,
                currency: trade_row.price_currency,
            },
            fee: MonetaryAmount {
                amount: trade_row.fee,
                currency: trade_row.fee_currency,
            },
            executed_date: trade_row.executed_date,
            provider: trade_row.provider,
            provider_id: trade_row.provider_id,
        }
    }
}
pub fn list_trades(db: &Db) -> Result<Vec<Trade>, anyhow::Error> {
    let mut statement = db.connection.prepare(SELECT_SQL)?;

    from_rows::<TradeRow>(statement.query([])?)
        .map(|r| {
            let row = r.map_err(anyhow::Error::from)?;
            Ok(Trade::from(row))
        })
        .collect::<Result<Vec<_>, _>>()
}
