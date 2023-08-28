//! GeoJSON conversions.
pub(crate) mod geojson_line_reader;
pub(crate) mod geojson_reader;
pub(crate) mod geojson_writer;

pub use geojson_line_reader::*;
pub use geojson_reader::*;
pub use geojson_writer::*;

pub(crate) mod conversion {
    use crate::error::Result;
    use crate::geojson::GeoJsonWriter;
    use crate::{GeozeroDatasource, GeozeroGeometry};

    /// Convert to GeoJSON.
    pub trait ToJson {
        /// Convert to GeoJSON String.
        fn to_json(&self) -> Result<String>;
    }

    impl<T: GeozeroGeometry> ToJson for T {
        fn to_json(&self) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            let mut p = GeoJsonWriter::new(&mut out);
            self.process_geom(&mut p)?;
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }

    /// Consume features as GeoJSON.
    pub trait ProcessToJson {
        /// Consume features as GeoJSON String.
        fn to_json(&mut self) -> Result<String>;
    }

    impl<T: GeozeroDatasource> ProcessToJson for T {
        fn to_json(&mut self) -> Result<String> {
            let mut out: Vec<u8> = Vec::new();
            let mut p = GeoJsonWriter::new(&mut out);
            self.process(&mut p)?;
            String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }
}

impl From<geojson::Error> for crate::error::GeozeroError {
    fn from(geojson_error: geojson::Error) -> Self {
        match geojson_error {
            geojson::Error::Io(io) => Self::IoError(io),
            other => Self::Geometry(other.to_string()),
        }
    }
}

#[cfg(feature = "with-wkb")]
mod wkb {
    use crate::error::Result;
    use crate::geojson::{GeoJsonString, GeoJsonWriter};
    use crate::wkb::{FromWkb, WkbDialect};
    use std::io::Read;

    impl FromWkb for GeoJsonString {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut out: Vec<u8> = Vec::new();
            let mut p = GeoJsonWriter::new(&mut out);
            crate::wkb::process_wkb_type_geom(rdr, &mut p, dialect)?;
            let json = String::from_utf8(out).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })?;
            Ok(GeoJsonString(json))
        }
    }

    // crate::impl_postgres_postgis_decode!(GeoJsonString);
    // crate::impl_postgres_postgis_encode!(GeoJsonString);
    // crate::impl_sqlx_postgis_type_info!(GeoJsonString);
    // crate::impl_sqlx_postgis_decode!(GeoJsonString);
    // crate::impl_sqlx_postgis_encode!(GeoJsonString);
    // crate::impl_sqlx_gpkg_type_info!(GeoJsonString);
    // crate::impl_sqlx_gpkg_decode!(GeoJsonString);
    // crate::impl_sqlx_gpkg_encode!(GeoJsonString);
}
