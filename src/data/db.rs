use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::{fs, path::PathBuf};

use crate::error::AppError;

pub enum DbLocation {
    Persisted,
    #[cfg(test)]
    InMemory,
}

pub struct Db {
    pub connection: Connection,
}

impl Db {
    // TODO: Split into getting connection and migration
    pub fn open(location: &DbLocation) -> Result<Self, AppError> {
        let connection = match location {
            DbLocation::Persisted => {
                let data_dir = get_data_directory()?;
                fs::create_dir_all(&data_dir)?;

                let db_path = data_dir.join("invporis.db");

                // If a database does not exist at the path, one is created.
                Connection::open(&db_path)?
            }
            #[cfg(test)]
            DbLocation::InMemory => Connection::open_in_memory()?,
        };

        connection.execute(
            "CREATE TABLE IF NOT EXISTS trades (
             id INTEGER PRIMARY KEY,
             event TEXT NOT NULL,
             isin TEXT NULL,
             quantity TEXT NOT NULL,
             price TEXT NOT NULL,
             executed_at INTEGER NOT NULL,
             currency TEXT NOT NULL DEFAULT 'USD',
             fee TEXT NOT NULL DEFAULT '0',
             provider TEXT NULL,
             provider_id TEXT NULL);
         ",
            (),
        )?;

        Ok(Self { connection })
    }
}

fn get_data_directory() -> Result<PathBuf, AppError> {
    ProjectDirs::from("io", "casperolsen", "invporis")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or(AppError::Path(
            "Could not determine data directory".to_string(),
        ))
}
