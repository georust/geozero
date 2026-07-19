//! Encode geozero features into MapLibre Tiles (MLT).
//!
//! Builds `mlt-core`'s row-oriented [`TileLayer`] directly from the event stream, with no MVT intermediate.

use std::collections::HashMap;
use std::mem;

use mlt_core::encoder::EncoderConfig;
use mlt_core::geo_types::{
    Coord, Geometry, LineString, MultiLineString, MultiPoint, MultiPolygon, Point, Polygon,
};
use mlt_core::{PropKind, PropValue, PropertyKey, TileLayer};

use super::mlt_error::MltError;
use crate::error::Result;
use crate::geo_types::GeoWriter;
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
    layer_name: String,
    extent: u32,
    geom: GeoWriter,
    columns: Columns,
    features: Vec<BufferedFeature>,
    current_props: Vec<(usize, PropValue)>,
}

struct BufferedFeature {
    geometry: Geometry<i32>,
    props: Vec<(usize, PropValue)>,
}

/// The union of property columns seen across features.
///
/// geozero streams properties per-feature by name, but MLT wants columns declared once and referenced positionally.
/// This assigns each name a stable index in first-appearance order.
#[derive(Default)]
struct Columns {
    defs: Vec<(String, PropKind)>,
    index: HashMap<String, usize>,
}

impl Columns {
    /// Index of the column named `name`, creating it (typed by `kind`) on first use.
    /// A later value of a disagreeing type is not reconciled here; the encoder rejects it in [`MltWriter::finish`].
    fn get_or_insert(&mut self, name: &str, kind: PropKind) -> usize {
        if let Some(&idx) = self.index.get(name) {
            return idx;
        }
        let idx = self.defs.len();
        self.defs.push((name.to_owned(), kind));
        self.index.insert(name.to_owned(), idx);
        idx
    }
}

impl MltWriter {
    /// Create a writer for one MLT layer with the given name and tile extent.
    pub fn new(layer_name: impl Into<String>, extent: u32) -> Result<Self> {
        Ok(Self {
            layer_name: layer_name.into(),
            extent,
            geom: GeoWriter::new(),
            columns: Columns::default(),
            features: Vec::new(),
            current_props: Vec::new(),
        })
    }

    /// Finish encoding and return the MLT tile bytes.
    pub fn finish(self) -> Result<Vec<u8>> {
        let mut builder =
            TileLayer::builder(&self.layer_name, self.extent).map_err(MltError::from)?;

        // Declare every column up front so each feature can back-fill nulls.
        let keys: Vec<PropertyKey> = self
            .columns
            .defs
            .iter()
            .map(|(name, kind)| builder.add_property(name.as_str(), *kind))
            .collect::<mlt_core::MltResult<_>>()
            .map_err(MltError::from)?;

        for feature in self.features {
            let mut fb = builder.feature(feature.geometry);
            for (col, value) in feature.props {
                fb.property(keys[col], value).map_err(MltError::from)?;
            }
            fb.finish().map_err(MltError::from)?;
        }

        let layer = builder.finish();
        Ok(layer
            .encode(EncoderConfig::default())
            .map_err(MltError::from)?)
    }
}

impl TryFrom<&ColumnValue<'_>> for PropValue {
    type Error = MltError;

    fn try_from(value: &ColumnValue<'_>) -> std::result::Result<Self, MltError> {
        Ok(match *value {
            ColumnValue::Bool(v) => PropValue::Bool(Some(v)),
            ColumnValue::Byte(v) => PropValue::I8(Some(v)),
            ColumnValue::UByte(v) => PropValue::U8(Some(v)),
            ColumnValue::Short(v) => PropValue::I32(Some(i32::from(v))),
            ColumnValue::UShort(v) => PropValue::U32(Some(u32::from(v))),
            ColumnValue::Int(v) => PropValue::I32(Some(v)),
            ColumnValue::UInt(v) => PropValue::U32(Some(v)),
            ColumnValue::Long(v) => PropValue::I64(Some(v)),
            ColumnValue::ULong(v) => PropValue::U64(Some(v)),
            ColumnValue::Float(v) => PropValue::F32(Some(v)),
            ColumnValue::Double(v) => PropValue::F64(Some(v)),
            ColumnValue::String(v) => PropValue::Str(Some(v.to_string())),
            ColumnValue::Json(v) => PropValue::Str(Some(v.to_string())),
            ColumnValue::DateTime(v) => PropValue::Str(Some(v.to_string())),
            ColumnValue::Binary(_) => return Err(MltError::UnsupportedColumnValue("binary")),
        })
    }
}

fn coord_i32(c: Coord<f64>) -> Coord<i32> {
    Coord {
        x: c.x as i32,
        y: c.y as i32,
    }
}

fn line_i32(line: &LineString<f64>) -> LineString<i32> {
    LineString(line.0.iter().copied().map(coord_i32).collect())
}

fn polygon_i32(polygon: &Polygon<f64>) -> Polygon<i32> {
    Polygon::new(
        line_i32(polygon.exterior()),
        polygon.interiors().iter().map(line_i32).collect(),
    )
}

/// `GeometryCollection` and curve variants have no MLT form and are rejected.
fn geometry_i32(geometry: Geometry<f64>) -> Result<Geometry<i32>> {
    Ok(match geometry {
        Geometry::Point(p) => Geometry::Point(Point(coord_i32(p.0))),
        Geometry::MultiPoint(mp) => Geometry::MultiPoint(MultiPoint(
            mp.0.iter().map(|p| Point(coord_i32(p.0))).collect(),
        )),
        Geometry::LineString(ls) => Geometry::LineString(line_i32(&ls)),
        Geometry::MultiLineString(mls) => {
            Geometry::MultiLineString(MultiLineString(mls.0.iter().map(line_i32).collect()))
        }
        Geometry::Polygon(poly) => Geometry::Polygon(polygon_i32(&poly)),
        Geometry::MultiPolygon(mp) => {
            Geometry::MultiPolygon(MultiPolygon(mp.0.iter().map(polygon_i32).collect()))
        }
        _ => return Err(MltError::UnsupportedGeometry.into()),
    })
}

impl GeomProcessor for MltWriter {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.geom.xy(x, y, idx)
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.geom.point_begin(idx)
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.geom.point_end(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom.multipoint_begin(size, idx)
    }
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.geom.multipoint_end(idx)
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geom.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.geom.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.geom.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geom.polygon_begin(tagged, size, idx)
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.geom.polygon_end(tagged, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom.multipolygon_begin(size, idx)
    }
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.geom.multipolygon_end(idx)
    }
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geom.geometrycollection_begin(size, idx)
    }
    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.geom.geometrycollection_end(idx)
    }
}

impl PropertyProcessor for MltWriter {
    fn property(&mut self, _idx: usize, name: &str, value: &ColumnValue) -> Result<bool> {
        let value = PropValue::try_from(value)?;
        let col = self.columns.get_or_insert(name, value.kind());
        self.current_props.push((col, value));
        Ok(true)
    }
}

impl FeatureProcessor for MltWriter {
    fn feature_begin(&mut self, _idx: u64) -> Result<()> {
        self.current_props.clear();
        Ok(())
    }
    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        let geometry = self.geom.take_geometry().ok_or(MltError::MissingGeometry)?;
        self.features.push(BufferedFeature {
            geometry: geometry_i32(geometry)?,
            props: mem::take(&mut self.current_props),
        });
        Ok(())
    }
}
