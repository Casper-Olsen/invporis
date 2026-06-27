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
    pub fn open() -> Result<Db, DataError> {
        let mut path = get_data_directory()?;
        path.push("invporis.db");

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        // If a database does not exist at the path, one is created.
        let conn = Connection::open(&path)?;

        Ok(Self { connection: conn })
    }
}

fn get_data_directory() -> Result<PathBuf, DataError> {
    ProjectDirs::from("com", "Casper-Olsen", "invporis")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or(DataError::Path("Could not determine data directory"))
}
