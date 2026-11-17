use thiserror::Error;
use wkt::error::ParseError;

/// CSV Error type.
#[derive(Error, Debug)]
pub enum CsvError {
    #[error("column not found or null")]
    ColumnNotFound,
    #[error("error parsing to WKT `{0}`")]
    WktError(#[source] ParseError),
}
