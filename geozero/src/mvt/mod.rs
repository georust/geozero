//! MVT (Mapbox Vector Tile) conversions.
//!
//! Reading and writing is built on the [`fast-mvt`](https://crates.io/crates/fast-mvt)
//! crate. Geometries and features are represented with its high-level, integer-only
//! (tile-space) types rather than raw protobuf messages:
//!
//! * [`MvtTile`], [`MvtLayer`], [`MvtFeature`] — owned, inspectable tile/layer/feature types.
//! * [`MvtValue`] — a typed property value.
//! * [`MvtReaderRef`] — a zero-copy decoder for encoded tile bytes.
//!
//! Encode an [`MvtTile`] with [`MvtTile::encode`], and decode bytes with
//! [`MvtReaderRef::new`] (use [`MvtReaderRef::to_tile`] for an owned [`MvtTile`]).

pub use fast_mvt::{
    DEFAULT_EXTENT, MvtExtent, MvtFeature, MvtGeometry, MvtLayer, MvtReaderRef, MvtTile, MvtValue,
};

pub(crate) mod mvt_reader;
pub(crate) mod mvt_writer;

pub use mvt_reader::*;
pub use mvt_writer::*;

pub(crate) mod conversion {
    use crate::GeozeroGeometry;
    use crate::error::Result;
    use crate::mvt::MvtWriter;
    use fast_mvt::MvtFeature;

    /// Convert to an MVT [`MvtFeature`].
    pub trait ToMvt {
        /// Convert to an MVT feature, transforming geometries into tile coordinate space.
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
        ) -> Result<MvtFeature>;

        /// Convert to an MVT feature with geometries in unmodified tile coordinate space.
        fn to_mvt_unscaled(&self) -> Result<MvtFeature>;
    }

    impl<T: GeozeroGeometry> ToMvt for T {
        fn to_mvt(
            &self,
            extent: u32,
            left: f64,
            bottom: f64,
            right: f64,
            top: f64,
        ) -> Result<MvtFeature> {
            let mut mvt = MvtWriter::new(extent, left, bottom, right, top)?;
            self.process_geom(&mut mvt)?;
            mvt.into_feature()
        }

        fn to_mvt_unscaled(&self) -> Result<MvtFeature> {
            // unwrap is safe since extent is nonzero
            // note that extent does not matter here since
            // only layers have extent values, not features
            let mut mvt = MvtWriter::new_unscaled(4096).unwrap();
            self.process_geom(&mut mvt)?;
            mvt.into_feature()
        }
    }
}

mod mvt_error;
pub use mvt_error::MvtError;

#[cfg(feature = "with-wkb")]
mod wkb {
    use std::io::Read;

    use crate::error::Result;
    use crate::mvt::MvtWriter;
    use crate::wkb::{FromWkb, WkbDialect};
    use fast_mvt::MvtFeature;

    impl FromWkb for MvtFeature {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            // unwrap is safe since extent is nonzero
            // note that extent does not matter here since
            // only layers have extent values, not features
            let mut mvt = MvtWriter::new_unscaled(4096).unwrap();
            crate::wkb::process_wkb_type_geom(rdr, &mut mvt, dialect)?;
            mvt.into_feature()
        }
    }
}
