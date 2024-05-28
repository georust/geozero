//! CSV Error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum CsvError {
    #[error("column not found or null")]
    ColumnNotFound,
    #[error("Invalid UTF-8 encoding")]
    InvalidUtf8,
    #[error("error processing dataset: `{0}`")]
    Processing(String),
    #[error("error parsing to WKT `{0}`")]
    WktError(String),
}
