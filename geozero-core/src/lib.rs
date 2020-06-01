//! Collection of GeoZero API implementations.

#[cfg(feature = "gdal-lib")]
mod gdal_reader;
#[cfg(feature = "gdal-lib")]
mod gdal_writer;
mod geojson_reader;
mod geojson_writer;
#[cfg(feature = "geos-lib")]
mod geos_reader;
#[cfg(feature = "geos-lib")]
mod geos_writer;
mod rustgeo_writer;
/// SVG Writer.
pub mod svg;
mod wkb_reader;
mod wkt_writer;

/// GeoJSON Reader + Writer.
pub mod geojson {
    pub use crate::geojson_reader::*;
    pub use crate::geojson_writer::*;
}

/// [georust/geo](https://github.com/georust/geo) Writer.
pub mod geo {
    pub use crate::rustgeo_writer::*;
}

/// WKB Reader.
pub mod wkb {
    pub use crate::wkb_reader::*;
}

/// WKT Writer.
pub mod wkt {
    pub use crate::wkt_writer::*;
}

/// [GEOS](https://github.com/georust/geos) Reader + Writer.
#[cfg(feature = "geos-lib")]
pub mod geos {
    pub use crate::geos_reader::*;
    pub use crate::geos_writer::*;
}

/// [GDAL](https://github.com/georust/gdal) Reader + Writer.
#[cfg(feature = "gdal-lib")]
pub mod gdal {
    pub use crate::gdal_reader::*;
    pub use crate::gdal_writer::*;
}
