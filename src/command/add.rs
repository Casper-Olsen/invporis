use colored::Colorize;
use rust_decimal::{Decimal, prelude::Zero};

use crate::{
    cli::command::{AddCryptoArgs, AddEquityArgs},
    data::{db::Db, trade_store},
    domain::trade::{CryptoTrade, EquityTrade, MonetaryAmount, Trade},
};

pub fn run_for_equity(args: AddEquityArgs, db: &Db) -> Result<(), anyhow::Error> {
    let trade = EquityTrade {
        event: crate::domain::trade::Event::from(args.event.clone()),
        isin: args.isin,
        symbol: args.symbol,
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

    trade_store::insert_trade(db, &Trade::Equity(trade))?;

    eprintln!(
        "Added equity trade with event: {}",
        args.event.to_string().green()
    );

    Ok(())
}

pub fn run_for_crypto(args: AddCryptoArgs, db: &Db) -> Result<(), anyhow::Error> {
    let trade = CryptoTrade {
        event: crate::domain::trade::Event::from(args.event.clone()),
        symbol: args.symbol,
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

    trade_store::insert_trade(db, &Trade::Crypto(trade))?;

    eprintln!(
        "Added crypto trade with event: {}",
        args.event.to_string().green()
    );

    Ok(())
}
