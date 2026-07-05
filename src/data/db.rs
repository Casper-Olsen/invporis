use directories::ProjectDirs;
use rusqlite::{Connection, Result, params};
use std::{fs, path::PathBuf};
use thiserror::Error;

use crate::cli;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Path error")]
    Path(&'static str),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

#[derive(Clone, Debug)]
pub enum Event {
    Buy,
}

impl rusqlite::ToSql for Event {
    fn to_sql(&self) -> Result<rusqlite::types::ToSqlOutput<'_>> {
        let e = match self {
            Self::Buy => "buy",
        };

        Ok(rusqlite::types::ToSqlOutput::from(e))
    }
}

impl From<cli::command::Event> for Event {
    fn from(item: cli::command::Event) -> Self {
        match item {
            cli::command::Event::Buy => Self::Buy,
        }
    }
}

#[derive(Debug)]
pub struct Trade {
    pub event: Event,
    pub symbol: String,
}

pub struct Db {
    connection: Connection,
}

impl Db {
    pub fn open() -> Result<Self, DataError> {
        let data_dir = get_data_directory()?;
        fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("invporis.db");

        // If a database does not exist at the path, one is created.
        let connection = Connection::open(&db_path)?;

        connection.execute(
            "CREATE TABLE IF NOT EXISTS trades (
             id INTEGER PRIMARY KEY,
             event TEXT NOT NULL,
             symbol TEXT NOT NULL
         )",
            (),
        )?;

        Ok(Self { connection })
    }

    pub fn insert_trade(&self, trade: &Trade) -> Result<(), DataError> {
        self.connection.execute(
            "insert into trades (event, symbol) values (?1, ?2)",
            params![trade.event, trade.symbol],
        )?;

        Ok(())
    }
}

fn get_data_directory() -> Result<PathBuf, DataError> {
    ProjectDirs::from("io", "casperolsen", "invporis")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or(DataError::Path("Could not determine data directory"))
}
