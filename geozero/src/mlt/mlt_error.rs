//! MLT error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MltError {
    /// Error from the `mlt-core` decoder or encoder.
    #[error("MLT codec error: {0}")]
    Codec(#[from] mlt_core::MltError),
    /// A geometry variant that MLT cannot represent.
    #[error("unsupported geometry type for MLT")]
    UnsupportedGeometry,
    /// A feature reached `feature_end` without any geometry.
    #[error("feature has no geometry")]
    MissingGeometry,
    /// A property value type that MLT cannot represent.
    #[error("unsupported property value type for MLT: {0}")]
    UnsupportedColumnValue(&'static str),
}
