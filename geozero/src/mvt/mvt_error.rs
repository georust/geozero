//! MVT error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MvtError {
    #[error("invalid extent, extent cannot be 0")]
    InvalidExtent,
    /// A ring needs at least 3 distinct coordinates, a line at least 2.
    #[error("too few coordinates in line or ring")]
    TooFewCoordinates,
    /// An error originating from the underlying `fast-mvt` crate.
    #[error(transparent)]
    Mvt(#[from] fast_mvt::MvtError),
}
