//! Error and Result types.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeozeroError {
    // Geometry access errors
    #[error("spatial index access")]
    GeometryIndex,
    #[error("geometry format")]
    GeometryFormat,
    // Http errors
    #[error("http status {0}")]
    HttpStatus(u16),
    #[error("http error `{0}`")]
    HttpError(String),
    // FeatureProcessor
    #[error("processing dataset: `{0}`")]
    Dataset(String),
    #[error("processing feature: `{0}`")]
    Feature(String),
    #[error("processing properties: `{0}`")]
    Properties(String),
    #[error("processing feature geometry: `{0}`")]
    FeatureGeometry(String),
    // PropertyProcessor
    #[error("processing feature property: `{0}`")]
    Property(String),
    #[error("column not found or null")]
    ColumnNotFound,
    #[error("expected a `{0}` value but found `{1}`")]
    ColumnType(String, String),
    // GeometryProcessor
    #[error("accessing requested coordinate")]
    Coord,
    #[error("invalid SRID value `{0}`")]
    Srid(i32),
    #[error("processing geometry `{0}`")]
    Geometry(String),
    // General
    #[error("I/O error `{0}`")]
    IoError(#[from] std::io::Error),
    #[cfg(feature = "with-mvt")]
    #[error("MVT error `{0}`")]
    MvtError(#[from] crate::mvt::MvtError),
    #[cfg(feature = "with-gdal")]
    #[error("GDAL error `{0}`")]
    GdalError(#[from] crate::gdal::GdalError),
}

pub type Result<T> = std::result::Result<T, GeozeroError>;
