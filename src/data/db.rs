use directories::ProjectDirs;
use rusqlite::{Connection, Result};
use std::{fs, path::PathBuf};

use crate::apperror::AppError;

pub struct Db {
    pub connection: Connection,
}

#[derive(Clone, Copy)]
pub enum DbLocation {
    Persisted,
    #[cfg(test)]
    InMemory,
}

impl Db {
    pub fn open(location: DbLocation) -> Result<Self, AppError> {
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
