//! GDAL error type.
use gdal_sys::OGRwkbGeometryType;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum GdalError {
    #[error("Unsupported geometry type: {0}")]
    UnsupportedGeometryType(OGRwkbGeometryType::Type),
}
