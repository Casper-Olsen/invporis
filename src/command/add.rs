use colored::Colorize;

use crate::{
    cli::command::AddArgs,
    data::{db::Db, trade_store},
    domain::trade::{MonetaryAmount, Trade},
    error::AppError,
};

pub fn run(args: AddArgs, db: &Db) -> Result<(), AppError> {
    println!(
        "Adding trade with event: {}",
        args.event.to_string().green()
    );

    let trade = Trade {
        event: crate::domain::trade::Event::from(args.event),
        isin: args.isin,
        quantity: args.quantity,
        price: MonetaryAmount {
            amount: args.price,
            currency: args.price_currency,
        },
        executed_at: args.executed_at,
        fee: MonetaryAmount {
            amount: args.fee,
            currency: args.fee_currency,
        },
        provider: None,
        provider_id: None,
    };

    trade_store::insert_trade(db, &trade)?;

    Ok(())
}
