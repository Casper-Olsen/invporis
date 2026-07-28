use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Path error: {0}")]
    Path(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Xslx error: {0}")]
    Xslx(#[from] calamine::XlsxError),
}
