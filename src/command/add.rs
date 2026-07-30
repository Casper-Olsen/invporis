use colored::Colorize;

use crate::{
    cli::command::AddArgs,
    data::{db::Db, trade_store},
    domain::trade::Trade,
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
        price: args.price,
        executed_at: args.executed_at,
        currency: args.currency,
        fee: args.fee,
        provider: None,
        provider_id: None,
    };

    trade_store::insert_trade(db, &trade)?;

    Ok(())
}
