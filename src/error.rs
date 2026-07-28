use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Path error: {0}")]
    Path(String),

    #[error("Import error: {0}")]
    ImportError(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
}
