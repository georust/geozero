//! MLT error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MltError {
    /// Error from the `mlt-core` decoder or encoder.
    #[error("MLT codec error: {0}")]
    Codec(#[from] mlt_core::MltError),
    /// The intermediate MVT tile failed to encode.
    #[error("MVT encoding error: {0}")]
    MvtEncode(#[from] prost::EncodeError),
    /// A geometry variant that MLT cannot represent.
    #[error("unsupported geometry type for MLT")]
    UnsupportedGeometry,
}
