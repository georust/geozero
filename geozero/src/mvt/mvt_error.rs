//! MVT error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MvtError {
    #[error("invalid feature.tags length: {0}")]
    InvalidFeatureTagsLength(usize),
    #[error("invalid key index {0}")]
    InvalidKeyIndex(usize),
    #[error("invalid value index {0}")]
    InvalidValueIndex(usize),
    #[error("unsupported value type for key {0}")]
    UnsupportedKeyValueType(String),
    #[error("geometry format")]
    GeometryFormat,
    #[error("Too few coordinates in line or ring")]
    TooFewCoordinates,
}
