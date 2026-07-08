use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::{fs, path::PathBuf};

use crate::error::AppError;

pub struct Db {
    pub connection: Connection,
}

impl Db {
    pub fn open() -> Result<Self, AppError> {
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
             fee TEXT NOT NULL DEFAULT '0'
         )",
            (),
        )?;

        Ok(Self { connection })
    }
}

fn get_data_directory() -> Result<PathBuf, AppError> {
    ProjectDirs::from("io", "casperolsen", "invporis")
        .map(|p| p.data_dir().to_path_buf())
        .ok_or(AppError::Path("Could not determine data directory"))
}
