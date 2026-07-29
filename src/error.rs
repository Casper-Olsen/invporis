use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Path error: {0}")]
    Path(String),

    #[error("Import error: {0}")]
    Import(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("XLSX error: {0}")]
    Xlsx(#[from] calamine::XlsxError),

    #[error("Csv error: {0}")]
    Csv(#[from] csv::Error),

    #[error("Calculation error: {0}")]
    Calculation(String),
}
