use anyhow::anyhow;
use colored::Colorize;
use rust_decimal::{Decimal, prelude::Zero};

use crate::{
    cli::command::AddArgs,
    data::{db::Db, trade_store},
    domain::trade::{MonetaryAmount, Trade},
};

pub fn run(args: AddArgs, db: &Db) -> Result<(), anyhow::Error> {
    if args.isin.is_none() && args.symbol.is_none() {
        return Err(anyhow!("Either ISIN or Symbol must be provided"));
    }

    let trade = Trade {
        event: crate::domain::trade::Event::from(args.event.clone()),
        isin: args.isin,
        asset_type: args.asset_type.into(),
        symbol: None,
        quantity: args.quantity,
        price: MonetaryAmount {
            amount: args.price,
            currency: args.price_currency,
        },
        executed_date: args.executed_date,
        fee: MonetaryAmount {
            amount: if args.fee.is_zero() {
                Decimal::zero()
            } else {
                // We always want the fee as a negative value
                -args.fee
            },
            currency: args.fee_currency,
        },
        provider: None,
        provider_id: None,
    };

    trade_store::insert_trade(db, &trade)?;

    eprintln!("Added trade with event: {}", args.event.to_string().green());

    Ok(())
}
