mod header;
mod point_z;
mod property_processor;
pub mod reader;
mod shp_reader;
mod shx_reader;

pub use crate::header::ShapeType;
pub use crate::property_processor::*;
pub use crate::reader::Reader;
pub use crate::shp_reader::NO_DATA;

// Re-export GeoZero to help avoid version conflicts
pub use geozero;

/// All Errors that can happen when using this library
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Wrapper around standard io::Error that might occur when reading/writing
    #[error("I/O error")]
    IoError(#[from] std::io::Error),
    /// The file read had an invalid File code (meaning it's not a Shapefile)
    #[error("The code `{0}` does not correspond to any of the ShapeType code defined by ESRI")]
    InvalidFileCode(i32),
    /// The file read had an invalid [ShapeType](enum.ShapeType.html) code
    /// (either in the file header or any record type)
    #[error("The file code `{0}` is invalid, is this a Shapefile?")]
    InvalidShapeType(i32),
    /// The Multipatch shape read from the file had an invalid [PatchType](enum.PatchType.html) code
    #[error("Invalid patch type `{0}`")]
    InvalidPatchType(i32),
    /// Error returned when trying to read the shape records as a certain shape type
    /// but the actual shape type does not correspond to the one asked
    #[error("The requested type: '{requested}' does not correspond to the actual shape type: '{actual}'")]
    MismatchShapeType {
        /// The requested ShapeType
        requested: ShapeType,
        /// The actual type of the shape
        actual: ShapeType,
    },
    #[error("Invalid shape record size")]
    InvalidShapeRecordSize,
    #[error("Dbase Error")]
    DbaseError(#[from] dbase::Error),
    #[error("Dbf missing")]
    MissingDbf,
    #[error("Index file missing")]
    MissingIndexFile,
    #[error("Geozero error")]
    GeozeroError(#[from] geozero::error::GeozeroError),
}
