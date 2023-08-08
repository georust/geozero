//! MVT conversions.
mod mvt_commands;
pub(crate) mod mvt_reader;
pub(crate) mod mvt_writer;

mod tag_builder;
pub use tag_builder::TagsBuilder;

mod tile_value;
pub use tile_value::TileValue;

#[rustfmt::skip]
mod vector_tile;

pub use mvt_reader::*;
pub use mvt_writer::*;
pub use prost::Message;
pub use vector_tile::*;

pub(crate) mod conversion {
    use crate::error::Result;
    use crate::mvt::vector_tile::tile;
    use crate::mvt::MvtWriter;
    use crate::GeozeroGeometry;

    /// Convert to MVT geometry.
    pub trait ToMvt {
        /// Convert to MVT geometry.
        ///
        /// # Arguments
        /// * `extent` - Size of MVT tile in tile coordinate space (e.g. 4096).
        /// * `left`, `bottom`, `right`, `top` - Bounds of tile in map coordinate space, with no buffer.
        fn to_mvt(
            &self,
            extent: u32,
            left: f64,
            bottom: f64,
            right: f64,
            top: f64,
        ) -> Result<tile::Feature>;
        /// Convert to MVT geometry with geometries in tile coordinate space.
        fn to_mvt_raw(&self) -> Result<tile::Feature>;
    }

    impl<T: GeozeroGeometry> ToMvt for T {
        fn to_mvt(
            &self,
            extent: u32,
            left: f64,
            bottom: f64,
            right: f64,
            top: f64,
        ) -> Result<tile::Feature> {
            let mut mvt = MvtWriter::new(extent, left, bottom, right, top);
            self.process_geom(&mut mvt)?;
            Ok(mvt.feature)
        }
        fn to_mvt_raw(&self) -> Result<tile::Feature> {
            let mut mvt = MvtWriter::default();
            self.process_geom(&mut mvt)?;
            Ok(mvt.feature)
        }
    }
}

mod mvt_error;
pub use mvt_error::MvtError;

#[cfg(feature = "with-wkb")]
mod wkb {
    use crate::error::Result;
    use crate::mvt::vector_tile::tile;
    use crate::mvt::MvtWriter;
    use crate::wkb::{FromWkb, WkbDialect};
    use std::io::Read;

    impl FromWkb for tile::Feature {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut mvt = MvtWriter::default();
            crate::wkb::process_wkb_type_geom(rdr, &mut mvt, dialect)?;
            Ok(mvt.feature)
        }
    }
}
