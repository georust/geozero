//! MVT error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MvtError {
    #[error("invalid feature.tags length: {0}")]
    InvalidFeatureTagsLength(usize),
    #[error("invalid key index {0}")]
    InvalidKeyIndex(u32),
    #[error("invalid value index {0}")]
    InvalidValueIndex(u32),
    #[error("unsupported value type for key {0}")]
    UnsupportedKeyValueType(String),
    #[error("geometry format")]
    GeometryFormat,
    #[error("too few coordinates in line or ring")]
    TooFewCoordinates,
}
