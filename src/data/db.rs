use directories::ProjectDirs;
use rusqlite::{Connection, Result, params};
use std::{fs, path::PathBuf};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataError {
    #[error("Path error")]
    Path(&'static str),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}

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

pub struct Trade {
    pub event: Event,
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
        let conn = Connection::open(&db_path)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS trades (
             id INTEGER PRIMARY KEY,
             event TEXT NOT NULL
         )",
            (),
        )?;

        Ok(Self { connection: conn })
    }

    pub fn insert_trade(&self, trade: &Trade) -> Result<(), DataError> {
        self.connection.execute(
            "insert into trades (event) values (?1)",
            params![trade.event],
        )?;

        Ok(())
    }
}

fn get_data_directory() -> Result<PathBuf, DataError> {
    ProjectDirs::from("com", "Casper-Olsen", "invporis")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or(DataError::Path("Could not determine data directory"))
}
