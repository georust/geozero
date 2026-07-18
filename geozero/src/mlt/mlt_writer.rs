//! Encode geozero features into MapLibre Tiles (MLT).
//!
//! geozero has no native MLT encoder. This writer builds an MVT layer with
//! [`MvtWriter`], then bridges it to MLT via `mlt-core`.

use mlt_core::encoder::EncoderConfig;

use super::mlt_error::MltError;
use crate::error::Result;
use crate::mvt::{Message, MvtWriter, Tile};
use crate::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};

/// Writer producing a single-layer MapLibre Tiles (MLT) tile.
///
/// Geometries are expected in tile-local coordinates matching `extent`.
/// Process a datasource, then call [`finish`](MltWriter::finish) for the bytes.
///
/// # Example
/// ```
/// use geozero::mlt::MltWriter;
/// use geozero::geojson::GeoJson;
/// use geozero::GeozeroDatasource;
///
/// # fn run() -> geozero::error::Result<()> {
/// let mut geojson = GeoJson(
///     r#"{ "type": "Point", "coordinates": [10, 20] }"#,
/// );
/// let mut writer = MltWriter::new("layer", 4096)?;
/// geojson.process(&mut writer)?;
/// let mlt_bytes: Vec<u8> = writer.finish()?;
/// # Ok(())
/// # }
/// # run().unwrap();
/// ```
pub struct MltWriter {
    mvt: MvtWriter,
    layer_name: String,
}

impl MltWriter {
    /// Create a writer for one MLT layer with the given name and tile extent.
    pub fn new(layer_name: impl Into<String>, extent: u32) -> Result<Self> {
        Ok(Self {
            mvt: MvtWriter::new_unscaled(extent)?,
            layer_name: layer_name.into(),
        })
    }

    /// Finish encoding and return the MLT tile bytes.
    pub fn finish(self) -> Result<Vec<u8>> {
        // Build an MVT tile from the accumulated features.
        let layer = self.mvt.layer(&self.layer_name);
        let tile = Tile {
            layers: vec![layer],
        };
        let mut mvt_bytes = Vec::new();
        tile.encode(&mut mvt_bytes).map_err(MltError::from)?;

        // Bridge MVT bytes -> TileLayer -> MLT bytes.
        let tile_layers = mlt_core::mvt::mvt_to_tile_layers(&mvt_bytes).map_err(MltError::from)?;
        let mut out = Vec::new();
        for layer in tile_layers {
            out.extend(
                layer
                    .encode(EncoderConfig::default())
                    .map_err(MltError::from)?,
            );
        }
        Ok(out)
    }
}

// Forward every method MvtWriter overrides to the inner writer.
// Unlisted methods are no-ops in both, so the defaults stay in sync.
impl GeomProcessor for MltWriter {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.mvt.xy(x, y, idx)
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.mvt.point_begin(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.mvt.multipoint_begin(size, idx)
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.mvt.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.mvt.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.mvt.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.mvt.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.mvt.polygon_begin(tagged, size, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.mvt.multipolygon_begin(size, idx)
    }
}

impl PropertyProcessor for MltWriter {
    fn property(&mut self, idx: usize, name: &str, value: &ColumnValue) -> Result<bool> {
        self.mvt.property(idx, name, value)
    }
}

impl FeatureProcessor for MltWriter {
    fn feature_begin(&mut self, idx: u64) -> Result<()> {
        self.mvt.feature_begin(idx)
    }
    fn feature_end(&mut self, idx: u64) -> Result<()> {
        self.mvt.feature_end(idx)
    }
    fn geometry_begin(&mut self) -> Result<()> {
        self.mvt.geometry_begin()
    }
    fn geometry_end(&mut self) -> Result<()> {
        self.mvt.geometry_end()
    }
}
