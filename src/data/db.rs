use directories::ProjectDirs;
use rusqlite::{Connection, Result};
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

pub struct Db {
    pub connection: Connection,
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
             symbol TEXT NOT NULL,
             quantity TEXT NOT NULL,
             price TEXT NOT NULL,
             executed_at INTEGER NOT NULL,
             currency TEXT NOT NULL DEFAULT 'USD',
             commission TEXT NOT NULL DEFAULT '0'
         )",
            (),
        )?;

        Ok(Self { connection })
    }
}

fn get_data_directory() -> Result<PathBuf, DataError> {
    ProjectDirs::from("io", "casperolsen", "invporis")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or(DataError::Path("Could not determine data directory"))
}
