//! MVT conversions.
pub(crate) mod mvt_reader;
// pub(crate) mod mvt_writer;
mod vector_tile;

pub use mvt_reader::*;
// pub use mvt_writer::*;

// pub(crate) mod conversion {
//     use super::mvt_writer::*;
//     use crate::error::Result;
//     use crate::GeozeroGeometry;

//     /// Convert to MVT geometry.
//     pub trait ToMvt {
//         /// Convert to MVT geometry.
//         fn to_mvt(&self) -> Result<tile::Feature>;
//     }

//     impl<T: GeozeroGeometry> ToMvt for T {
//         fn to_mvt(&self) -> Result<tile::Feature> {
//             let mut mvt = MvtWriter::new();
//             self.process_geom(&mut mvt)?;
//             Ok(mvt.geom)
//         }
//     }
// }

// #[cfg(feature = "with-wkb")]
// mod wkb {
//     use super::mvt_writer::*;
//     use crate::error::Result;
//     use crate::wkb::{FromWkb, WkbDialect};
//     use std::io::Read;

//     impl FromWkb for tile::Feature {
//         fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
//             let mut mvt = MvtWriter::new();
//             crate::wkb::process_wkb_type_geom(rdr, &mut mvt, dialect)?;
//             Ok(mvt.geom)
//         }
//     }
// }
