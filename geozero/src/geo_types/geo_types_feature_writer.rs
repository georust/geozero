use crate::error::{GeozeroError, Result};
use crate::geo_types::GeoWriter;
use crate::{
    ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, PropertyProcessor,
    PropertyReadType,
};
use std::collections::HashMap;

pub type GeoProperties = HashMap<String, OwnedColumnValue>;

#[derive(Debug, PartialEq)]
pub struct GeoFeature {
    geometry: geo_types::Geometry,
    properties: GeoProperties,
}

#[derive(Debug, Default)]
pub struct GeoFeatureWriter {
    geometry_writer: GeoWriter,
    next_properties: Option<GeoProperties>,
    pub(crate) features: Vec<GeoFeature>,
}

impl GeomProcessor for GeoFeatureWriter {
    fn dimensions(&self) -> CoordDimensions {
        self.geometry_writer.dimensions()
    }
    fn multi_dim(&self) -> bool {
        self.geometry_writer.multi_dim()
    }
    fn srid(&mut self, srid: Option<i32>) -> Result<()> {
        self.geometry_writer.srid(srid)
    }
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.geometry_writer.xy(x, y, idx)
    }
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) -> Result<()> {
        self.geometry_writer.coordinate(x, y, z, m, t, tm, idx)
    }
    fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.empty_point(idx)
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.point_begin(idx)
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.point_end(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.multipoint_begin(size, idx)
    }
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.multipoint_end(idx)
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.linestring_begin(tagged, size, idx)
    }
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.geometry_writer.linestring_end(tagged, idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.multilinestring_begin(size, idx)
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.multilinestring_end(idx)
    }
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.polygon_begin(tagged, size, idx)
    }
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.geometry_writer.polygon_end(tagged, idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.multipolygon_begin(size, idx)
    }
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.multipolygon_end(idx)
    }
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.geometrycollection_begin(size, idx)
    }
    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.geometrycollection_end(idx)
    }
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.circularstring_begin(size, idx)
    }
    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.circularstring_end(idx)
    }
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.compoundcurve_begin(size, idx)
    }
    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.compoundcurve_end(idx)
    }
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.curvepolygon_begin(size, idx)
    }
    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.curvepolygon_end(idx)
    }
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.multicurve_begin(size, idx)
    }
    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.multicurve_end(idx)
    }
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.multisurface_begin(size, idx)
    }
    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.multisurface_end(idx)
    }
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.triangle_begin(tagged, size, idx)
    }
    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        self.geometry_writer.triangle_end(tagged, idx)
    }
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.polyhedralsurface_begin(size, idx)
    }
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.polyhedralsurface_end(idx)
    }
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometry_writer.tin_begin(size, idx)
    }
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        self.geometry_writer.tin_end(idx)
    }
}

impl PropertyProcessor for GeoFeatureWriter {
    fn property(&mut self, _idx: usize, name: &str, value: &ColumnValue) -> Result<bool> {
        let properties = self
            .next_properties
            .as_mut()
            .expect("properties should be initialized before traversing properties");
        properties.insert(name.to_string(), OwnedColumnValue::from(value));
        Ok(true)
    }
}

/// Feature property value.
///
/// Like [`ColumnValue`], but owns it's data.
#[derive(Clone, PartialEq, Debug)]
pub enum OwnedColumnValue {
    Byte(i8),
    UByte(u8),
    Bool(bool),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
    Long(i64),
    ULong(u64),
    Float(f32),
    Double(f64),
    String(String),
    /// A JSON-formatted string
    Json(String),
    /// A datetime stored as an ISO8601-formatted string
    DateTime(String),
    Binary(Vec<u8>),
}

impl<'a> From<&'a OwnedColumnValue> for ColumnValue<'a> {
    fn from(value: &'a OwnedColumnValue) -> Self {
        match value {
            OwnedColumnValue::Byte(v) => ColumnValue::Byte(*v),
            OwnedColumnValue::UByte(v) => ColumnValue::UByte(*v),
            OwnedColumnValue::Bool(v) => ColumnValue::Bool(*v),
            OwnedColumnValue::Short(v) => ColumnValue::Short(*v),
            OwnedColumnValue::UShort(v) => ColumnValue::UShort(*v),
            OwnedColumnValue::Int(v) => ColumnValue::Int(*v),
            OwnedColumnValue::UInt(v) => ColumnValue::UInt(*v),
            OwnedColumnValue::Long(v) => ColumnValue::Long(*v),
            OwnedColumnValue::ULong(v) => ColumnValue::ULong(*v),
            OwnedColumnValue::Float(v) => ColumnValue::Float(*v),
            OwnedColumnValue::Double(v) => ColumnValue::Double(*v),
            OwnedColumnValue::String(v) => ColumnValue::String(v),
            OwnedColumnValue::Json(v) => ColumnValue::Json(v),
            OwnedColumnValue::DateTime(v) => ColumnValue::DateTime(v),
            OwnedColumnValue::Binary(v) => ColumnValue::Binary(v),
        }
    }
}

impl From<&ColumnValue<'_>> for OwnedColumnValue {
    fn from(value: &ColumnValue) -> Self {
        match value {
            ColumnValue::Byte(v) => OwnedColumnValue::Byte(*v),
            ColumnValue::UByte(v) => OwnedColumnValue::UByte(*v),
            ColumnValue::Bool(v) => OwnedColumnValue::Bool(*v),
            ColumnValue::Short(v) => OwnedColumnValue::Short(*v),
            ColumnValue::UShort(v) => OwnedColumnValue::UShort(*v),
            ColumnValue::Int(v) => OwnedColumnValue::Int(*v),
            ColumnValue::UInt(v) => OwnedColumnValue::UInt(*v),
            ColumnValue::Long(v) => OwnedColumnValue::Long(*v),
            ColumnValue::ULong(v) => OwnedColumnValue::ULong(*v),
            ColumnValue::Float(v) => OwnedColumnValue::Float(*v),
            ColumnValue::Double(v) => OwnedColumnValue::Double(*v),
            ColumnValue::String(str) => OwnedColumnValue::String(str.to_string()),
            ColumnValue::Json(str) => OwnedColumnValue::Json(str.to_string()),
            ColumnValue::DateTime(str) => OwnedColumnValue::DateTime(str.to_string()),
            ColumnValue::Binary(bytes) => OwnedColumnValue::Binary(bytes.to_vec()),
        }
    }
}

impl FeatureProcessor for GeoFeatureWriter {
    fn feature_begin(&mut self, _idx: u64) -> Result<()> {
        debug_assert!(self.geometry_writer.is_empty());
        debug_assert!(self.next_properties.is_none());
        self.next_properties = Some(GeoProperties::new());
        Ok(())
    }
    fn feature_end(&mut self, _idx: u64) -> Result<()> {
        let Some(geometry) = self.geometry_writer.take_geometry() else {
            return Err(GeozeroError::FeatureGeometry(
                "missing geometry".to_string(),
            ));
        };
        let Some(properties) = self.next_properties.take() else {
            return Err(GeozeroError::Feature("missing properties".to_string()));
        };

        self.features.push(GeoFeature {
            geometry,
            properties,
        });

        Ok(())
    }
    fn geometry_begin(&mut self) -> Result<()> {
        debug_assert!(self.geometry_writer.is_empty());
        Ok(())
    }
}

impl GeoFeatureWriter {
    pub(crate) fn new() -> Self {
        Self::default()
    }
}

impl GeoFeature {
    pub fn geometry(&self) -> &geo_types::Geometry {
        &self.geometry
    }

    pub fn geometry_mut(&mut self) -> &mut geo_types::Geometry {
        &mut self.geometry
    }

    pub fn properties(&self) -> &GeoProperties {
        &self.properties
    }

    pub fn properties_mut(&mut self) -> &mut GeoProperties {
        &mut self.properties
    }

    pub fn property<T: PropertyReadType>(&self, name: &str) -> Result<T> {
        let Some(owned_column_value) = self.properties.get(name) else {
            return Err(GeozeroError::ColumnNotFound);
        };

        let column_value = ColumnValue::from(owned_column_value);
        T::get_value(&column_value)
    }

    pub fn into_inner(self) -> (geo_types::Geometry, GeoProperties) {
        (self.geometry, self.properties)
    }
}

#[cfg(test)]
mod tests {
    use crate::geojson::GeoJsonReader;
    use crate::ToGeoFeatures;
    use std::fs::File;
    #[test]
    fn from_json() {
        let f = File::open("tests/data/places.json").unwrap();
        let mut geojson = GeoJsonReader(f);
        let feature_iter = geojson.to_geo_features().unwrap();
        let features: Vec<_> = feature_iter.collect();
        let first = features.first().unwrap();
        assert_eq!(first.property::<String>("NAME").unwrap(), "Bombo");
        assert_eq!(
            first.geometry(),
            &geo_types::Geometry::Point(geo_types::Point::new(
                32.533299524864844,
                0.583299105614628
            ))
        );

        let last = features.last().unwrap();
        assert_eq!(last.property::<String>("NAME").unwrap(), "Hong Kong");
        assert_eq!(
            last.geometry(),
            &geo_types::Geometry::Point(geo_types::Point::new(
                114.18306345846304,
                22.30692675357551
            ))
        );
    }
}
