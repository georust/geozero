//! SVG conversions.
mod writer;
pub use writer::SvgWriter;

/// SVG String.
pub struct SvgString(pub String);

pub(crate) mod conversion {
    use super::*;
    use crate::error::Result;
    use crate::FeatureProcessor;
    use crate::{GeozeroDatasource, GeozeroGeometry};

    /// Convert to SVG.
    ///
    /// # Usage example:
    ///
    /// Convert a geo-types `Polygon` to an SVG document:
    ///
    /// ```
    /// use geozero::ToSvg;
    /// use geo_types::polygon;
    ///
    /// let geom: geo_types::Geometry<f64> = polygon![
    ///     (x: 220., y: 10.),
    ///     (x: 300., y: 210.),
    ///     (x: 170., y: 250.),
    ///     (x: 123., y: 234.),
    /// ]
    /// .into();
    ///
    /// println!("{}", &geom.to_svg_document().unwrap());
    /// ```
    pub trait ToSvg {
        /// Convert to SVG geometry.
        fn to_svg(&self) -> Result<String>;
        /// Convert to SVG document.
        fn to_svg_document(&self) -> Result<String>;
    }

    impl<T: GeozeroGeometry> ToSvg for T {
        fn to_svg(&self) -> Result<String> {
            let mut svg_data: Vec<u8> = Vec::new();
            let mut svg = SvgWriter::new(&mut svg_data, false);
            self.process_geom(&mut svg)?;
            String::from_utf8(svg_data).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
        fn to_svg_document(&self) -> Result<String> {
            let mut svg_data: Vec<u8> = Vec::new();
            let mut svg = SvgWriter::new(&mut svg_data, false);
            // svg.set_dimensions(bbox.get(0), bbox.get(1), bbox.get(2), bbox.get(3), 800, 400);
            svg.dataset_begin(None)?;
            svg.feature_begin(0)?;
            self.process_geom(&mut svg)?;
            svg.feature_end(0)?;
            svg.dataset_end()?;
            String::from_utf8(svg_data).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }

    /// Consume features as SVG.
    pub trait ProcessToSvg {
        /// Consume features as SVG String.
        fn to_svg(&mut self) -> Result<String>;
    }

    impl<T: GeozeroDatasource> ProcessToSvg for T {
        fn to_svg(&mut self) -> Result<String> {
            let mut svg_data: Vec<u8> = Vec::new();
            let mut svg = SvgWriter::new(&mut svg_data, false);
            self.process(&mut svg)?;
            String::from_utf8(svg_data).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })
        }
    }
}

#[cfg(feature = "with-wkb")]
mod wkb {
    use super::*;
    use crate::error::Result;
    use crate::wkb::{FromWkb, WkbDialect};
    use std::io::Read;

    impl FromWkb for SvgString {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut svg_data: Vec<u8> = Vec::new();
            let mut writer = SvgWriter::new(&mut svg_data, false);
            crate::wkb::process_wkb_type_geom(rdr, &mut writer, dialect)?;
            let svg = String::from_utf8(svg_data).map_err(|_| {
                crate::error::GeozeroError::Geometry("Invalid UTF-8 encoding".to_string())
            })?;
            Ok(SvgString(svg))
        }
    }
}
