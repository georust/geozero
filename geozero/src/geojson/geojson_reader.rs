use crate::error::Result;
use crate::{
    ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource, GeozeroGeometry,
    PropertyProcessor,
};
use geojson::{Feature, FeatureReader};
use geojson::{GeoJson as GeoGeoJson, Geometry, Value};
use serde_json::map::Map;
use serde_json::value::Value as JsonValue;
use std::io::Read;

/// GeoJSON String.
#[derive(Debug)]
pub struct GeoJsonString(pub String);

impl GeozeroGeometry for GeoJsonString {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        read_geojson_geom(&mut self.0.as_bytes(), processor)
    }
}

impl GeozeroDatasource for GeoJsonString {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        read_geojson(&mut self.0.as_bytes(), processor)
    }
}

/// GeoJSON String slice.
pub struct GeoJson<'a>(pub &'a str);

impl GeozeroGeometry for GeoJson<'_> {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        read_geojson_geom(&mut self.0.as_bytes(), processor)
    }
}

impl GeozeroDatasource for GeoJson<'_> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        read_geojson(&mut self.0.as_bytes(), processor)
    }
}

/// GeoJSON Reader.
pub struct GeoJsonReader<R: Read>(pub R);

impl<R: Read> GeozeroDatasource for GeoJsonReader<R> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        read_geojson(&mut self.0, processor)
    }
}

/// Read and process GeoJSON.
pub fn read_geojson<R: Read, P: FeatureProcessor>(mut reader: R, processor: &mut P) -> Result<()> {
    let mut geojson_str = String::new();
    reader.read_to_string(&mut geojson_str)?;
    let geojson = geojson_str.parse::<GeoGeoJson>()?;
    process_geojson(&geojson, processor)
}

pub fn read_geojson_fc<R: Read, P: FeatureProcessor>(reader: R, processor: &mut P) -> Result<()> {
    for (idx, feature) in FeatureReader::from_reader(reader).features().enumerate() {
        process_geojson_feature(&feature?, idx, processor)?;
    }

    Ok(())
}

/// Read and process GeoJSON geometry.
pub fn read_geojson_geom<R: Read, P: GeomProcessor>(
    reader: &mut R,
    processor: &mut P,
) -> Result<()> {
    let mut geojson_str = String::new();
    reader.read_to_string(&mut geojson_str)?;
    let geojson = geojson_str.parse::<GeoGeoJson>()?;
    process_geojson_geom(&geojson, processor)
}

/// Process top-level GeoJSON items
fn process_geojson<P: FeatureProcessor>(gj: &GeoGeoJson, processor: &mut P) -> Result<()> {
    match *gj {
        GeoGeoJson::FeatureCollection(ref collection) => {
            processor.dataset_begin(None)?;
            for (idx, feature) in collection.features.iter().enumerate() {
                processor.feature_begin(idx as u64)?;
                if let Some(ref properties) = feature.properties {
                    processor.properties_begin()?;
                    process_properties(properties, processor)?;
                    processor.properties_end()?;
                }
                if let Some(ref geometry) = feature.geometry {
                    processor.geometry_begin()?;
                    process_geojson_geom_n(geometry, idx, processor)?;
                    processor.geometry_end()?;
                }
                processor.feature_end(idx as u64)?;
            }
            processor.dataset_end()
        }
        GeoGeoJson::Feature(ref feature) => process_geojson_feature(feature, 0, processor),
        GeoGeoJson::Geometry(ref geometry) => process_geojson_geom_n(geometry, 0, processor),
    }
}

/// Process top-level GeoJSON items
fn process_geojson_feature<P: FeatureProcessor>(
    feature: &Feature,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.dataset_begin(None)?;
    if feature.geometry.is_some() || feature.properties.is_some() {
        processor.feature_begin(idx as u64)?;
        if let Some(ref properties) = feature.properties {
            processor.properties_begin()?;
            process_properties(properties, processor)?;
            processor.properties_end()?;
        }
        if let Some(ref geometry) = feature.geometry {
            processor.geometry_begin()?;
            process_geojson_geom_n(geometry, idx, processor)?;
            processor.geometry_end()?;
        }
        processor.feature_end(idx as u64)?;
    }
    processor.dataset_end()
}

/// Process top-level GeoJSON items (geometry only)
fn process_geojson_geom<P: GeomProcessor>(gj: &GeoGeoJson, processor: &mut P) -> Result<()> {
    match *gj {
        GeoGeoJson::FeatureCollection(ref collection) => {
            for (idx, geometry) in collection
                .features
                .iter()
                // Only pass on non-empty geometries, doing so by reference
                .filter_map(|feature| feature.geometry.as_ref())
                .enumerate()
            {
                process_geojson_geom_n(geometry, idx, processor)?;
            }
        }
        GeoGeoJson::Feature(ref feature) => {
            if let Some(ref geometry) = feature.geometry {
                process_geojson_geom_n(geometry, 0, processor)?;
            }
        }
        GeoGeoJson::Geometry(ref geometry) => {
            process_geojson_geom_n(geometry, 0, processor)?;
        }
    }
    Ok(())
}

/// Process GeoJSON geometries
pub(crate) fn process_geojson_geom_n<P: GeomProcessor>(
    geom: &Geometry,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    match geom.value {
        Value::Point(ref geometry) => {
            processor.point_begin(idx)?;
            process_coord(geometry, processor.multi_dim(), 0, processor)?;
            processor.point_end(idx)
        }
        Value::MultiPoint(ref geometry) => {
            processor.multipoint_begin(geometry.len(), idx)?;
            let multi_dim = processor.multi_dim();
            for (idxc, point_type) in geometry.iter().enumerate() {
                process_coord(point_type, multi_dim, idxc, processor)?;
            }
            processor.multipoint_end(idx)
        }
        Value::LineString(ref geometry) => process_linestring(geometry, true, idx, processor),
        Value::MultiLineString(ref geometry) => {
            processor.multilinestring_begin(geometry.len(), idx)?;
            for (idx2, linestring_type) in geometry.iter().enumerate() {
                process_linestring(linestring_type, false, idx2, processor)?;
            }
            processor.multilinestring_end(idx)
        }
        Value::Polygon(ref geometry) => process_polygon(geometry, true, idx, processor),
        Value::MultiPolygon(ref geometry) => {
            processor.multipolygon_begin(geometry.len(), idx)?;
            for (idx2, polygon_type) in geometry.iter().enumerate() {
                process_polygon(polygon_type, false, idx2, processor)?;
            }
            processor.multipolygon_end(idx)
        }
        Value::GeometryCollection(ref collection) => {
            processor.geometrycollection_begin(collection.len(), idx)?;
            for (idx2, geometry) in collection.iter().enumerate() {
                process_geojson_geom_n(geometry, idx2, processor)?;
            }
            processor.geometrycollection_end(idx)
        }
    }
}

/// Process GeoJSON properties
pub(crate) fn process_properties<P: PropertyProcessor>(
    properties: &Map<String, JsonValue>,
    processor: &mut P,
) -> Result<()> {
    for (i, (key, value)) in properties.iter().enumerate() {
        // Could we provide a stable property index?
        match value {
            JsonValue::String(v) => processor.property(i, key, &ColumnValue::String(v))?,
            JsonValue::Number(v) if v.is_f64() => {
                processor.property(i, key, &ColumnValue::Double(v.as_f64().unwrap()))?
            }
            JsonValue::Number(v) if v.is_i64() => {
                processor.property(i, key, &ColumnValue::Long(v.as_i64().unwrap()))?
            }
            JsonValue::Number(v) if v.is_u64() => {
                processor.property(i, key, &ColumnValue::ULong(v.as_u64().unwrap()))?
            }
            JsonValue::Bool(v) => processor.property(i, key, &ColumnValue::Bool(*v))?,
            // Null, Array(Vec<Value>), Object(Map<String, Value>)
            _ => processor.property(i, key, &ColumnValue::String(&value.to_string()))?,
        };
    }
    Ok(())
}

type Position = Vec<f64>;
type PointType = Position;
type LineStringType = Vec<Position>;
type PolygonType = Vec<Vec<Position>>;

fn process_coord<P: GeomProcessor>(
    point_type: &PointType,
    multi_dim: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    if multi_dim {
        processor.coordinate(
            point_type[0],
            point_type[1],
            point_type.get(2).copied(),
            None,
            None,
            None,
            idx,
        )
    } else {
        processor.xy(point_type[0], point_type[1], idx)
    }
}

fn process_linestring<P: GeomProcessor>(
    linestring_type: &LineStringType,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.linestring_begin(tagged, linestring_type.len(), idx)?;
    let multi_dim = processor.multi_dim();
    for (idxc, point_type) in linestring_type.iter().enumerate() {
        process_coord(point_type, multi_dim, idxc, processor)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_polygon<P: GeomProcessor>(
    polygon_type: &PolygonType,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.polygon_begin(tagged, polygon_type.len(), idx)?;
    for (idx2, linestring_type) in polygon_type.iter().enumerate() {
        process_linestring(linestring_type, false, idx2, processor)?;
    }
    processor.polygon_end(tagged, idx)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geojson::GeoJsonWriter;
    use crate::wkt::WktWriter;
    use crate::{CoordDimensions, ProcessToSvg, ToJson, ToWkt};
    use std::fs::File;

    #[test]
    fn line_string() -> Result<()> {
        let geojson = r#"{
            "type": "LineString",
            "coordinates": [
                [1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]
            ]}"#;
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(
            read_geojson_geom(&mut geojson.as_bytes(), &mut WktWriter::new(&mut wkt_data)).is_ok()
        );
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(wkt, "LINESTRING(1875038.447610231 -3269648.6879248763,1874359.641504197 -3270196.812984864,1874141.0428635243 -3270953.7840121365,1874440.1778162003 -3271619.4315206874,1876396.0598222911 -3274138.747656357,1876442.0805243007 -3275052.60551469,1874739.312657555 -3275457.333765534)"
    );
        Ok(())
    }

    #[test]
    fn geometries3d() -> Result<()> {
        let geojson = r#"{"type": "LineString", "coordinates": [[1,1,10],[2,2,20]]}"#;
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut out = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyz());
        assert!(read_geojson_geom(&mut geojson.as_bytes(), &mut out).is_ok());
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(wkt, "LINESTRING(1 1 10,2 2 20)");

        let geojson = r#"{"type": "LineString", "coordinates": [[1,1],[2,2]]}"#;
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut out = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyz());
        assert!(read_geojson_geom(&mut geojson.as_bytes(), &mut out).is_ok());
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(wkt, "LINESTRING(1 1,2 2)");

        Ok(())
    }

    #[test]
    fn feature_collection() -> Result<()> {
        let geojson = r#"{
            "type": "FeatureCollection",
            "name": "countries",
            "features": [{
                "type": "Feature",
                "properties": {
                    "id": "NZL",
                    "name": "New Zealand"
                },
                "geometry": {
                    "type": "MultiPolygon",
                    "coordinates": [[[
                        [173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]
                    ]],[[
                        [174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]
                    ]]]
                }
            }]
        }"#;
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(read_geojson_fc(geojson.as_bytes(), &mut WktWriter::new(&mut wkt_data)).is_ok());
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(wkt, "MULTIPOLYGON(((173.020375 -40.919052,173.247234 -41.331999,173.958405 -40.926701,174.247587 -41.349155,174.248517 -41.770008,173.876447 -42.233184,173.22274 -42.970038,172.711246 -43.372288,173.080113 -43.853344,172.308584 -43.865694,171.452925 -44.242519,171.185138 -44.897104,170.616697 -45.908929,169.831422 -46.355775,169.332331 -46.641235,168.411354 -46.619945,167.763745 -46.290197,166.676886 -46.219917,166.509144 -45.852705,167.046424 -45.110941,168.303763 -44.123973,168.949409 -43.935819,169.667815 -43.555326,170.52492 -43.031688,171.12509 -42.512754,171.569714 -41.767424,171.948709 -41.514417,172.097227 -40.956104,172.79858 -40.493962,173.020375 -40.919052)),((174.612009 -36.156397,175.336616 -37.209098,175.357596 -36.526194,175.808887 -36.798942,175.95849 -37.555382,176.763195 -37.881253,177.438813 -37.961248,178.010354 -37.579825,178.517094 -37.695373,178.274731 -38.582813,177.97046 -39.166343,177.206993 -39.145776,176.939981 -39.449736,177.032946 -39.879943,176.885824 -40.065978,176.508017 -40.604808,176.01244 -41.289624,175.239567 -41.688308,175.067898 -41.425895,174.650973 -41.281821,175.22763 -40.459236,174.900157 -39.908933,173.824047 -39.508854,173.852262 -39.146602,174.574802 -38.797683,174.743474 -38.027808,174.697017 -37.381129,174.292028 -36.711092,174.319004 -36.534824,173.840997 -36.121981,173.054171 -35.237125,172.636005 -34.529107,173.007042 -34.450662,173.551298 -35.006183,174.32939 -35.265496,174.612009 -36.156397)))");
        Ok(())
    }

    #[test]
    fn properties() -> Result<()> {
        let geojson = r#"{
                "type": "Feature",
                "properties": {
                    "id": 1,
                    "name": "New Zealand"
                },
                "geometry": {
                    "type": "Point",
                    "coordinates": [10,20]
                }
            }"#;
        let mut geojson = GeoJson(geojson);
        let mut out: Vec<u8> = Vec::new();
        assert!(geojson.process(&mut GeoJsonWriter::new(&mut out)).is_ok());
        assert_eq!(
            std::str::from_utf8(&out).unwrap(),
            r#"{
"type": "FeatureCollection",
"features": [{"type": "Feature", "properties": {"id": 1, "name": "New Zealand"}, "geometry": {"type": "Point", "coordinates": [10,20]}}]}"#
        );

        assert_eq!(
            geojson.to_json().unwrap(),
            r#"{"type": "Point", "coordinates": [10,20]}"#
        );
        Ok(())
    }

    #[test]
    fn from_file() -> Result<()> {
        let f = File::open("tests/data/places.json")?;
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(read_geojson(f, &mut WktWriter::new(&mut wkt_data)).is_ok());
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(
            &wkt[0..100],
            "POINT(32.533299524864844 0.583299105614628),POINT(30.27500161597942 0.671004121125236),POINT(15.7989"
        );
        assert_eq!(
            &wkt[wkt.len()-100..],
            "06510862875),POINT(103.85387481909902 1.294979325105942),POINT(114.18306345846304 22.30692675357551)"
        );
        Ok(())
    }

    #[test]
    fn from_file_fc() -> Result<()> {
        let f = File::open("tests/data/places.json")?;
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(read_geojson_fc(f, &mut WktWriter::new(&mut wkt_data)).is_ok());
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(
            &wkt[0..100],
            "POINT(32.533299524864844 0.583299105614628),POINT(30.27500161597942 0.671004121125236),POINT(15.7989"
        );
        assert_eq!(
            &wkt[wkt.len()-100..],
            "06510862875),POINT(103.85387481909902 1.294979325105942),POINT(114.18306345846304 22.30692675357551)"
        );
        Ok(())
    }

    #[test]
    fn conversions() -> Result<()> {
        let geojson = GeoJson(r#"{"type": "Point", "coordinates": [10,20]}"#);
        assert_eq!(geojson.to_wkt().unwrap(), "POINT(10 20)");

        let mut f = File::open("tests/data/places.json")?;
        let svg = GeoJsonReader(&mut f).to_svg().unwrap();
        println!("{}", &svg[svg.len() - 100..]);
        assert_eq!(
            &svg[svg.len() - 100..],
            r#"387481909902 1.294979325105942 Z"/>
<path d="M 114.18306345846304 22.30692675357551 Z"/>
</g>
</svg>"#
        );

        Ok(())
    }
}
