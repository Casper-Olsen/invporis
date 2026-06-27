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

pub fn initialize() -> Result<(), DataError> {
    let mut path = get_data_directory()?;
    path.push("invporis.db");

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    // If a database does not exist at the path, one is created.
    let _conn = Connection::open(&path)?;

    Ok(())
}

fn get_data_directory() -> Result<PathBuf, DataError> {
    ProjectDirs::from("com", "Casper-Olsen", "invporis")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or(DataError::Path("Could not determine data directory"))
}
