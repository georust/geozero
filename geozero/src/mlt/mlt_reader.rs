//! Decode MapLibre Tiles (MLT) into geozero processors.

use mlt_core::geo_types::{Geometry, LineString, Polygon};
use mlt_core::{Decoder, LendingIterator, Parser, ParsedLayer, ParsedLayer01, PropValueRef};

use super::mlt_error::MltError;
use crate::error::Result;
use crate::{ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource};

/// Reader for a MapLibre Tiles (MLT) tile.
///
/// A tile holds one or more named layers. Each layer is a [`GeozeroDatasource`],
/// mirroring MVT. Geometries are emitted in raw tile-local coordinates.
///
/// # Example
/// ```no_run
/// use geozero::mlt::MltReader;
/// use geozero::geojson::GeoJsonWriter;
/// use geozero::GeozeroDatasource;
///
/// # fn run(tile_bytes: &[u8]) -> geozero::error::Result<()> {
/// let reader = MltReader::new(tile_bytes)?;
/// let mut out = Vec::new();
/// let mut json = GeoJsonWriter::new(&mut out);
/// for mut layer in reader.layers() {
///     layer.process(&mut json)?;
/// }
/// # Ok(())
/// # }
/// ```
pub struct MltReader<'a> {
    layers: Vec<ParsedLayer<'a>>,
}

impl<'a> MltReader<'a> {
    /// Parse and fully decode an MLT tile from its raw bytes.
    pub fn new(data: &'a [u8]) -> Result<Self> {
        let mut parser = Parser::default();
        let lazy = parser.parse_layers(data).map_err(MltError::from)?;
        let mut decoder = Decoder::default();
        let layers = decoder.decode_all(lazy).map_err(MltError::from)?;
        Ok(Self { layers })
    }

    /// Names of the MVT-compatible layers in this tile, in tile order.
    pub fn layer_names(&self) -> impl Iterator<Item = &str> + '_ {
        self.tag01_layers().map(|l| l.name())
    }

    /// The MVT-compatible layers in this tile, each a [`GeozeroDatasource`].
    pub fn layers(&self) -> impl Iterator<Item = MltLayerReader<'_, 'a>> + '_ {
        self.tag01_layers().map(|layer| MltLayerReader { layer })
    }

    /// Look up a single layer by name.
    pub fn layer(&self, name: &str) -> Option<MltLayerReader<'_, 'a>> {
        self.tag01_layers()
            .find(|l| l.name() == name)
            .map(|layer| MltLayerReader { layer })
    }

    /// The MVT-compatible (`Tag01`) layers; forward-compat `Unknown` layers are skipped.
    fn tag01_layers(&self) -> impl Iterator<Item = &ParsedLayer01<'a>> + '_ {
        self.layers.iter().filter_map(|layer| match layer {
            ParsedLayer::Tag01(l) => Some(l),
            _ => None,
        })
    }
}

/// A single MLT layer, usable as a [`GeozeroDatasource`] (one layer = one dataset).
pub struct MltLayerReader<'r, 'a> {
    layer: &'r ParsedLayer01<'a>,
}

impl<'a> MltLayerReader<'_, 'a> {
    /// The layer name.
    pub fn name(&self) -> &'a str {
        self.layer.name()
    }
}

impl GeozeroDatasource for MltLayerReader<'_, '_> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        process_layer(self.layer, processor)
    }
}

/// Process a single decoded MLT layer, driving `processor`.
///
/// Private so `mlt-core`'s `ParsedLayer01` stays out of geozero's public API.
fn process_layer<P: FeatureProcessor>(layer: &ParsedLayer01<'_>, processor: &mut P) -> Result<()> {
    processor.dataset_begin(Some(layer.name()))?;
    let mut idx: u64 = 0;
    let mut features = layer.iter_features();
    while let Some(feature) = features.next() {
        let feature = feature.map_err(MltError::from)?;

        processor.feature_begin(idx)?;

        processor.properties_begin()?;
        // iter_properties yields only present columns; nulls are skipped.
        for (i, column) in feature.iter_properties().enumerate() {
            let key = column.name().to_string();
            processor.property(i, &key, &to_column_value(column.value()))?;
        }
        processor.properties_end()?;

        processor.geometry_begin()?;
        process_geometry(feature.geometry(), 0, processor)?;
        processor.geometry_end()?;

        processor.feature_end(idx)?;
        idx += 1;
    }
    processor.dataset_end()
}

fn to_column_value(value: PropValueRef<'_>) -> ColumnValue<'_> {
    match value {
        PropValueRef::Bool(v) => ColumnValue::Bool(v),
        PropValueRef::I8(v) => ColumnValue::Byte(v),
        PropValueRef::U8(v) => ColumnValue::UByte(v),
        PropValueRef::I32(v) => ColumnValue::Int(v),
        PropValueRef::U32(v) => ColumnValue::UInt(v),
        PropValueRef::I64(v) => ColumnValue::Long(v),
        PropValueRef::U64(v) => ColumnValue::ULong(v),
        PropValueRef::F32(v) => ColumnValue::Float(v),
        PropValueRef::F64(v) => ColumnValue::Double(v),
        PropValueRef::Str(v) => ColumnValue::String(v),
    }
}

/// Emit a `geo_types::Geometry<i32>` (tile-local coordinates) as geozero events.
fn process_geometry<P: GeomProcessor>(
    geometry: &Geometry<i32>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    match geometry {
        Geometry::Point(p) => {
            processor.point_begin(idx)?;
            processor.xy(f64::from(p.x()), f64::from(p.y()), 0)?;
            processor.point_end(idx)?;
        }
        Geometry::MultiPoint(mp) => {
            processor.multipoint_begin(mp.0.len(), idx)?;
            for (i, p) in mp.0.iter().enumerate() {
                processor.xy(f64::from(p.x()), f64::from(p.y()), i)?;
            }
            processor.multipoint_end(idx)?;
        }
        Geometry::LineString(ls) => process_linestring(ls, true, idx, processor)?,
        Geometry::MultiLineString(mls) => {
            processor.multilinestring_begin(mls.0.len(), idx)?;
            for (i, ls) in mls.0.iter().enumerate() {
                process_linestring(ls, false, i, processor)?;
            }
            processor.multilinestring_end(idx)?;
        }
        Geometry::Polygon(poly) => process_polygon(poly, true, idx, processor)?,
        Geometry::MultiPolygon(mpoly) => {
            processor.multipolygon_begin(mpoly.0.len(), idx)?;
            for (i, poly) in mpoly.0.iter().enumerate() {
                process_polygon(poly, false, i, processor)?;
            }
            processor.multipolygon_end(idx)?;
        }
        _ => return Err(MltError::UnsupportedGeometry.into()),
    }
    Ok(())
}

fn process_linestring<P: GeomProcessor>(
    line: &LineString<i32>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.linestring_begin(tagged, line.0.len(), idx)?;
    for (i, coord) in line.0.iter().enumerate() {
        processor.xy(f64::from(coord.x), f64::from(coord.y), i)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_polygon<P: GeomProcessor>(
    polygon: &Polygon<i32>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let num_rings = 1 + polygon.interiors().len();
    processor.polygon_begin(tagged, num_rings, idx)?;
    process_linestring(polygon.exterior(), false, 0, processor)?;
    for (i, ring) in polygon.interiors().iter().enumerate() {
        process_linestring(ring, false, i + 1, processor)?;
    }
    processor.polygon_end(tagged, idx)
}
