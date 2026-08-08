use rusqlite_migration::{M, Migrations};

use crate::data::db::Db;

/// The list of sequential database migrations.
///
/// `rusqlite_migration` uses the `SQLite` internal `user_version` PRAGMA integer
/// to automatically track and apply these migrations in chronological order.
const MIGRATIONS_SLICE: &[M<'_>] = &[
    M::up(
        "CREATE TABLE trades (
             id INTEGER PRIMARY KEY,
             event TEXT NOT NULL,
             isin TEXT NULL,
             quantity TEXT NOT NULL,
             price TEXT NOT NULL,
             price_currency TEXT NOT NULL,
             fee TEXT NOT NULL DEFAULT '0',
             fee_currency TEXT NOT NULL,
             executed_date INTEGER NOT NULL,
             provider TEXT NULL,
             provider_id TEXT NULL);
         ",
    ),
    M::up(
        "CREATE TABLE securities (
             isin TEXT NOT NULL,
             currency TEXT NOT NULL,
             name TEXT NULL,
             PRIMARY KEY (isin, currency));
         ",
    ),
];

const MIGRATIONS: Migrations<'_> = Migrations::from_slice(MIGRATIONS_SLICE);

pub fn migrate(db: &mut Db) {
    // TODO: Don't use unwrap here
    MIGRATIONS.to_latest(&mut db.connection).unwrap();
}
