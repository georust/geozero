//! Encode geometries into MVT features and layers using the `fast-mvt` crate.
//! <https://github.com/mapbox/vector-tile-spec/tree/master/2.1>

use fast_mvt::{DEFAULT_EXTENT, MvtFeature, MvtLayer, MvtValue};
use geo_types::{
    Coord, Geometry, GeometryCollection, LineString, MultiLineString, MultiPoint, MultiPolygon,
    Point, Polygon,
};

use super::mvt_error::MvtError;
use crate::error::{GeozeroError, Result};
use crate::geometry_processor::bounded_vec;
use crate::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};

/// Generator for MVT geometries, features, and layers.
///
/// The writer implements the geozero processor traits, accumulating streamed
/// geometry (in tile coordinate space, as integer [`fast_mvt::MvtGeometry`]) and
/// properties into [`MvtFeature`]s. Call [`MvtWriter::layer`] to obtain the
/// finished [`MvtLayer`], which can be added to an
/// [`MvtTile`](crate::mvt::MvtTile) and encoded with
/// [`MvtTile::encode`](crate::mvt::MvtTile::encode).
///
/// # Example
/// Generate a MVT layer using a type that implements [`GeozeroDatasource`](crate::GeozeroDatasource).
/// ```
/// use geozero::{GeozeroDatasource, geojson::GeoJsonString, mvt::MvtWriter};
///
/// let mut geojson = GeoJsonString(
///     serde_json::json!({
///         "type": "FeatureCollection",
///         "features": [
///             {
///                 "type": "Feature",
///                 "properties": {
///                     "population": 100
///                 },
///                 "geometry": {
///                     "type": "Point",
///                     "coordinates": [1.0, 2.0]
///                 }
///             },
///             {
///                 "type": "Feature",
///                 "properties": {
///                     "population": 200
///                 },
///                 "geometry": {
///                     "type": "Point",
///                     "coordinates": [3.0, 4.0]
///                 }
///             }
///         ]
///     })
///     .to_string(),
/// ); // implements GeozeroDatasource
/// let mut mvt_writer = MvtWriter::new_unscaled(4096).unwrap();
/// geojson.process(&mut mvt_writer).unwrap();
/// let mvt_layer = mvt_writer.layer("sample"); // returns MVT layer with all features
/// ```
///
/// To convert a single geometry into a [`MvtFeature`], see the [`ToMvt`](crate::ToMvt) trait.
#[derive(Debug)]
pub struct MvtWriter {
    /// Completed features for the layer.
    features: Vec<MvtFeature>,
    /// Properties of the feature currently being processed.
    properties: Vec<(String, MvtValue)>,
    // Geometry accumulator (integer tile-space geo-types)
    geoms: Vec<Geometry<i32>>,
    /// Stack of any in-progress (potentially nested) `GeometryCollection`s
    collections: Vec<Vec<Geometry<i32>>>,
    /// In-progress multi-polygon
    polygons: Option<Vec<Polygon<i32>>>,
    /// In-progress polygon or multi-linestring
    line_strings: Option<Vec<LineString<i32>>>,
    /// In-progress point or line-string
    coords: Option<Vec<Coord<i32>>>,
    // Coordinate transformation
    extent: u32,
    scale: bool,
    left: f64,
    bottom: f64,
    x_multiplier: f64,
    y_multiplier: f64,
}

impl MvtWriter {
    /// Creates a new `MvtWriter` that transforms geometries to be in tile coordinate space.
    pub fn new(extent: u32, left: f64, bottom: f64, right: f64, top: f64) -> Result<MvtWriter> {
        if extent == 0 {
            return Err(MvtError::InvalidExtent.into());
        }
        Ok(MvtWriter {
            extent,
            scale: true,
            left,
            bottom,
            x_multiplier: f64::from(extent) / (right - left),
            y_multiplier: f64::from(extent) / (top - bottom),
            ..MvtWriter::empty(extent)
        })
    }

    /// Creates a new `MvtWriter` that does not transform any geometries.
    ///
    /// The resulting writer expects all geometries to be provided in tile coordinate space,
    /// matching the specified `extent`.
    pub fn new_unscaled(extent: u32) -> Result<MvtWriter> {
        if extent == 0 {
            return Err(MvtError::InvalidExtent.into());
        }
        Ok(MvtWriter::empty(extent))
    }

    fn empty(extent: u32) -> MvtWriter {
        MvtWriter {
            features: Vec::new(),
            properties: Vec::new(),
            geoms: Vec::new(),
            collections: Vec::new(),
            polygons: None,
            line_strings: None,
            coords: None,
            extent,
            scale: false,
            // unused unless scaling; see `transform`
            left: 0.0,
            bottom: 0.0,
            x_multiplier: 1.0,
            y_multiplier: 1.0,
        }
    }

    /// Consume the writer and return the last processed geometry as an [`MvtFeature`].
    ///
    /// Used by the [`ToMvt`](crate::ToMvt) trait to convert a single geometry.
    pub(crate) fn into_feature(mut self) -> Result<MvtFeature> {
        let geometry = self
            .take_geometry()
            .ok_or_else(|| GeozeroError::Geometry("Missing MVT geometry".to_string()))?;
        Ok(MvtFeature {
            id: None,
            geometry,
            properties: self.properties,
        })
    }

    /// Consume the writer and return an [`MvtLayer`] with all processed features.
    pub fn layer(self, name: &str) -> MvtLayer {
        MvtLayer {
            name: name.to_string(),
            // extent is nonzero (checked in `new`/`new_unscaled`)
            extent: fast_mvt::MvtExtent::new(self.extent).unwrap_or(DEFAULT_EXTENT),
            features: self.features,
        }
    }

    /// Transform a map-space coordinate into an integer tile-space coordinate.
    fn transform(&self, x: f64, y: f64) -> Coord<i32> {
        if self.scale {
            let x = ((x - self.left) * self.x_multiplier).floor() as i32;
            let y = ((y - self.bottom) * self.y_multiplier).floor() as i32;
            // Y is stored reversed in tile space
            Coord {
                x,
                y: (self.extent as i32).saturating_sub(y),
            }
        } else {
            Coord {
                x: x as i32,
                y: y as i32,
            }
        }
    }

    fn finish_geometry(&mut self, geometry: Geometry<i32>) -> Result<()> {
        if let Some(collection) = self.collections.last_mut() {
            collection.push(geometry);
        } else {
            self.geoms.push(geometry);
        }
        Ok(())
    }

    fn take_geometry(&mut self) -> Option<Geometry<i32>> {
        match self.geoms.len() {
            0 => None,
            1 => self.geoms.pop(),
            _ => Some(Geometry::GeometryCollection(GeometryCollection(
                std::mem::take(&mut self.geoms),
            ))),
        }
    }
}

impl GeomProcessor for MvtWriter {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        let coord = self.transform(x, y);
        let coords = self
            .coords
            .as_mut()
            .ok_or_else(|| GeozeroError::Geometry("Not ready for coords".to_string()))?;
        coords.push(coord);
        Ok(())
    }

    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        debug_assert!(self.coords.is_none());
        self.coords = Some(Vec::with_capacity(1));
        Ok(())
    }

    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let coords = self
            .coords
            .take()
            .ok_or_else(|| GeozeroError::Geometry("No coords for Point".to_string()))?;
        let coord = *coords
            .first()
            .ok_or_else(|| GeozeroError::Geometry("Empty Point".to_string()))?;
        self.finish_geometry(Point(coord).into())
    }

    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.coords.is_none());
        self.coords = Some(bounded_vec(size)?);
        Ok(())
    }

    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        let coords = self
            .coords
            .take()
            .ok_or_else(|| GeozeroError::Geometry("No coords for MultiPoint".to_string()))?;
        let points = coords.into_iter().map(Point::from).collect();
        self.finish_geometry(MultiPoint(points).into())
    }

    fn linestring_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.coords.is_none());
        self.coords = Some(bounded_vec(size)?);
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        let coords = self
            .coords
            .take()
            .ok_or_else(|| GeozeroError::Geometry("No coords for LineString".to_string()))?;
        let line_string = LineString(coords);
        if tagged {
            self.finish_geometry(line_string.into())
        } else {
            let line_strings = self.line_strings.as_mut().ok_or_else(|| {
                GeozeroError::Geometry("Missing container for LineString".to_string())
            })?;
            line_strings.push(line_string);
            Ok(())
        }
    }

    fn multilinestring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.line_strings.is_none());
        self.line_strings = Some(bounded_vec(size)?);
        Ok(())
    }

    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let line_strings = self.line_strings.take().ok_or_else(|| {
            GeozeroError::Geometry("No LineStrings for MultiLineString".to_string())
        })?;
        self.finish_geometry(MultiLineString(line_strings).into())
    }

    fn polygon_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.line_strings.is_none());
        self.line_strings = Some(bounded_vec(size)?);
        Ok(())
    }

    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        let mut line_strings = self
            .line_strings
            .take()
            .ok_or_else(|| GeozeroError::Geometry("Missing LineStrings for Polygon".to_string()))?;
        let polygon = if line_strings.is_empty() {
            Polygon::new(LineString(vec![]), vec![])
        } else {
            let exterior = line_strings.remove(0);
            Polygon::new(exterior, line_strings)
        };
        if tagged {
            self.finish_geometry(polygon.into())
        } else {
            let polygons = self.polygons.as_mut().ok_or_else(|| {
                GeozeroError::Geometry("Missing container for Polygon".to_string())
            })?;
            polygons.push(polygon);
            Ok(())
        }
    }

    fn multipolygon_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.polygons.is_none());
        self.polygons = Some(bounded_vec(size)?);
        Ok(())
    }

    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        let polygons = self.polygons.take().ok_or_else(|| {
            GeozeroError::Geometry("Missing polygons for MultiPolygon".to_string())
        })?;
        self.finish_geometry(MultiPolygon(polygons).into())
    }

    fn geometrycollection_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.collections.push(bounded_vec(size)?);
        Ok(())
    }

    fn geometrycollection_end(&mut self, _idx: usize) -> Result<()> {
        let geometries = self
            .collections
            .pop()
            .ok_or_else(|| GeozeroError::Geometry("Unexpected geometry type".to_string()))?;
        self.finish_geometry(Geometry::GeometryCollection(GeometryCollection(geometries)))
    }
}

impl PropertyProcessor for MvtWriter {
    fn property(&mut self, _idx: usize, name: &str, value: &ColumnValue) -> Result<bool> {
        self.properties
            .push((name.to_string(), column_value(value)));
        Ok(false)
    }
}

impl FeatureProcessor for MvtWriter {
    fn feature_begin(&mut self, _idx: u64) -> Result<()> {
        self.properties = Vec::new();
        Ok(())
    }

    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        if let Some(geometry) = self.take_geometry() {
            let properties = std::mem::take(&mut self.properties);
            self.features.push(MvtFeature {
                id: None,
                geometry,
                properties,
            });
        }
        Ok(())
    }
}

/// Convert a geozero [`ColumnValue`] into an MVT [`MvtValue`].
///
/// [`ColumnValue::Json`], [`ColumnValue::DateTime`], and [`ColumnValue::Binary`] are
/// stored as strings. For [`Binary`](ColumnValue::Binary), base64 encoding is used.
fn column_value(value: &ColumnValue) -> MvtValue {
    use base64::Engine;
    use base64::prelude::BASE64_STANDARD;

    match value {
        ColumnValue::Byte(v) => MvtValue::SInt(i64::from(*v)),
        ColumnValue::UByte(v) => MvtValue::UInt(u64::from(*v)),
        ColumnValue::Bool(v) => MvtValue::Bool(*v),
        ColumnValue::Short(v) => MvtValue::SInt(i64::from(*v)),
        ColumnValue::UShort(v) => MvtValue::UInt(u64::from(*v)),
        ColumnValue::Int(v) => MvtValue::SInt(i64::from(*v)),
        ColumnValue::UInt(v) => MvtValue::UInt(u64::from(*v)),
        ColumnValue::Long(v) => MvtValue::SInt(*v),
        ColumnValue::ULong(v) => MvtValue::UInt(*v),
        ColumnValue::Float(v) => MvtValue::Float(*v),
        ColumnValue::Double(v) => MvtValue::Double(*v),
        ColumnValue::String(v) => MvtValue::String((*v).to_string()),
        ColumnValue::Json(v) => MvtValue::String((*v).to_string()),
        ColumnValue::DateTime(v) => MvtValue::String((*v).to_string()),
        ColumnValue::Binary(v) => MvtValue::String(BASE64_STANDARD.encode(v)),
    }
}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use fast_mvt::{MvtGeometry, MvtValue};
    use geo_types::{Geometry, LineString, MultiLineString, MultiPolygon, Point, Polygon, point};
    use serde_json::json;

    use super::*;
    use crate::ToMvt;
    use crate::geojson::GeoJson;
    use crate::geojson::conversion::ToJson;

    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#435-example-geometry-encodings

    #[test]
    fn point_geom() {
        let geojson = GeoJson(r#"{"type": "Point", "coordinates": [25, 17]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(mvt.geometry, MvtGeometry::Point(point! { x: 25, y: 17 }));
    }

    #[test]
    fn multipoint_geom() {
        let geojson = GeoJson(r#"{"type": "MultiPoint", "coordinates": [[5, 7], [3, 2]]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            MvtGeometry::MultiPoint(vec![Point::new(5, 7), Point::new(3, 2)].into())
        );
    }

    #[test]
    fn line_geom() {
        let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[2,2], [2,10], [10,10]]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            MvtGeometry::LineString(LineString::from(vec![(2, 2), (2, 10), (10, 10)]))
        );
    }

    #[test]
    fn multiline_geom() {
        let geojson = GeoJson(
            r#"{"type": "MultiLineString", "coordinates": [[[2,2], [2,10], [10,10]],[[1,1],[3,5]]]}"#,
        );
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            MvtGeometry::MultiLineString(MultiLineString(vec![
                LineString::from(vec![(2, 2), (2, 10), (10, 10)]),
                LineString::from(vec![(1, 1), (3, 5)]),
            ]))
        );
    }

    #[test]
    fn polygon_geom() {
        let geojson =
            GeoJson(r#"{"type": "Polygon", "coordinates": [[[3, 6], [8, 12], [20, 34], [3, 6]]]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            MvtGeometry::Polygon(Polygon::new(
                LineString::from(vec![(3, 6), (8, 12), (20, 34), (3, 6)]),
                vec![],
            ))
        );
    }

    #[test]
    fn multipolygon_geom() {
        let geojson = r#"{
            "type": "MultiPolygon",
            "coordinates": [
                [[[0,0],[10,0],[10,10],[0,10],[0,0]]],
                [[[11,11],[20,11],[20,20],[11,20],[11,11]]]
            ]
        }"#;
        let mvt = GeoJson(geojson).to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            MvtGeometry::MultiPolygon(MultiPolygon(vec![
                Polygon::new(
                    LineString::from(vec![(0, 0), (10, 0), (10, 10), (0, 10), (0, 0)]),
                    vec![],
                ),
                Polygon::new(
                    LineString::from(vec![(11, 11), (20, 11), (20, 20), (11, 20), (11, 11)]),
                    vec![],
                ),
            ]))
        );
    }

    #[test]
    fn properties_are_collected() {
        use crate::GeozeroDatasource;

        // Properties are collected via the streaming datasource path (not `to_mvt`,
        // which only converts a single geometry).
        let mut geojson = GeoJson(
            r#"{"type": "Feature", "properties": {"name": "test", "count": 3},
                "geometry": {"type": "Point", "coordinates": [1, 2]}}"#,
        );
        let mut writer = MvtWriter::new_unscaled(4096).unwrap();
        geojson.process(&mut writer).unwrap();
        let layer = writer.layer("l");
        let feature = &layer.features[0];
        assert_eq!(feature.geometry, MvtGeometry::Point(point! { x: 1, y: 2 }));
        assert!(
            feature
                .properties
                .contains(&("name".to_string(), MvtValue::String("test".to_string())))
        );
        assert!(
            feature
                .properties
                .contains(&("count".to_string(), MvtValue::SInt(3)))
        );
    }

    #[test]
    #[cfg(feature = "with-geo")]
    fn geo_screen_coords_to_mvt() -> Result<()> {
        let geo: Geometry<f64> = Point::new(25.0, 17.0).into();
        let mvt = geo.to_mvt_unscaled()?;
        assert_eq!(mvt.geometry, MvtGeometry::Point(point! { x: 25, y: 17 }));
        Ok(())
    }

    #[test]
    #[cfg(feature = "with-geo")]
    fn geo_to_mvt() -> Result<()> {
        let geo: Geometry<f64> = Point::new(960000.0, 6002729.0).into();
        let mvt = geo.to_mvt(256, 958826.08, 5987771.04, 978393.96, 6007338.92)?;
        assert_eq!(mvt.geometry, MvtGeometry::Point(point! { x: 15, y: 61 }));
        let geojson = mvt.to_json()?;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({
                "type": "Point",
                "coordinates": [15,61]
            }) // without reverse_y: [15,195]
        );
        Ok(())
    }

    #[test]
    fn build_layer() {
        use crate::GeozeroDatasource;
        use crate::mvt::MvtTile;

        let mut geojson = GeoJson(
            r#"{"type": "FeatureCollection", "features": [
                {"type": "Feature", "properties": {"n": 1},
                 "geometry": {"type": "Point", "coordinates": [1, 2]}},
                {"type": "Feature", "properties": {"n": 2},
                 "geometry": {"type": "Point", "coordinates": [3, 4]}}
            ]}"#,
        );
        let mut writer = MvtWriter::new_unscaled(4096).unwrap();
        geojson.process(&mut writer).unwrap();
        let layer = writer.layer("sample");
        assert_eq!(layer.name, "sample");
        assert_eq!(layer.features.len(), 2);
        assert_eq!(
            layer.features[0].geometry,
            MvtGeometry::Point(point! { x: 1, y: 2 })
        );

        // Round-trips through encoding
        let mut tile = MvtTile::new();
        tile.add_layer(layer);
        assert!(!tile.encode().unwrap().is_empty());
    }
}
