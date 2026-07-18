//! MVT error type.
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MvtError {
    #[error("invalid extent, extent cannot be 0")]
    InvalidExtent,
    /// An error originating from the underlying `fast-mvt` crate.
    #[error(transparent)]
    Mvt(#[from] fast_mvt::MvtError),
}
