use thiserror::Error;

#[derive(Error, Debug)]
pub enum GeozeroError {
    // Geometry access errors
    #[error("spatial index access")]
    GeometryIndex,
    #[error("geometry format")]
    GeometryFormat,
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
    // GeometryProcessor
    #[error("accessing requested coordinate")]
    Coord,
    #[error("processing geometry `{0}`")]
    Geometry(String),
    // General
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, GeozeroError>;
