use geojson::{GeoJson, Geometry, Value};
use geozero::error::{GeozeroError, Result};
use geozero::{FeatureProcessor, GeomProcessor};
use std::io::Read;

/// Read and process GeoJSON
pub fn read_geojson<R: Read, P: FeatureProcessor>(mut reader: R, processor: &mut P) -> Result<()> {
    let mut geojson_str = String::new();
    reader.read_to_string(&mut geojson_str)?;
    let geojson = geojson_str
        .parse::<GeoJson>()
        .map_err(|_| GeozeroError::GeometryFormat)?;
    process_geojson(&geojson, processor)
}

/// Read and process GeoJSON geometry
pub fn read_geojson_geom<R: Read, P: GeomProcessor>(
    mut reader: R,
    processor: &mut P,
) -> Result<()> {
    let mut geojson_str = String::new();
    reader.read_to_string(&mut geojson_str)?;
    let geojson = geojson_str
        .parse::<GeoJson>()
        .map_err(|_| GeozeroError::GeometryFormat)?;
    process_geojson_geom(&geojson, processor)
}

/// Process top-level GeoJSON items
fn process_geojson<P: FeatureProcessor>(gj: &GeoJson, processor: &mut P) -> Result<()> {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => {
            processor.dataset_begin(None)?;
            for (idx, geometry) in collection
                .features
                .iter()
                // Only pass on non-empty geometries, doing so by reference
                .filter_map(|feature| feature.geometry.as_ref())
                .enumerate()
            {
                processor.feature_begin(idx as u64)?;
                processor.properties_begin()?;
                // TODO: properties
                processor.properties_end()?;
                processor.geometry_begin()?;
                match_geometry(geometry, idx, processor)?;
                processor.geometry_end()?;
                processor.feature_end(idx as u64)?;
            }
            processor.dataset_end()?;
        }
        GeoJson::Feature(ref feature) => {
            processor.dataset_begin(None)?;
            if let Some(ref geometry) = feature.geometry {
                processor.feature_begin(0)?;
                processor.properties_begin()?;
                // TODO: properties
                processor.properties_end()?;
                processor.geometry_begin()?;
                match_geometry(geometry, 0, processor)?;
                processor.geometry_end()?;
                processor.feature_end(0)?;
            }
            processor.dataset_end()?;
        }
        GeoJson::Geometry(ref geometry) => {
            match_geometry(geometry, 0, processor)?;
        }
    }
    Ok(())
}

/// Process top-level GeoJSON items (geometry only)
fn process_geojson_geom<P: GeomProcessor>(gj: &GeoJson, processor: &mut P) -> Result<()> {
    match *gj {
        GeoJson::FeatureCollection(ref collection) => {
            for (idx, geometry) in collection
                .features
                .iter()
                // Only pass on non-empty geometries, doing so by reference
                .filter_map(|feature| feature.geometry.as_ref())
                .enumerate()
            {
                match_geometry(geometry, idx, processor)?;
            }
        }
        GeoJson::Feature(ref feature) => {
            if let Some(ref geometry) = feature.geometry {
                match_geometry(geometry, 0, processor)?;
            }
        }
        GeoJson::Geometry(ref geometry) => {
            match_geometry(geometry, 0, processor)?;
        }
    }
    Ok(())
}

/// Process GeoJSON geometries
fn match_geometry<P: GeomProcessor>(geom: &Geometry, idx: usize, processor: &mut P) -> Result<()> {
    match geom.value {
        Value::Point(ref geometry) => {
            process_point(geometry, idx, processor)?;
        }
        Value::MultiPoint(ref geometry) => {
            process_multi_point(geometry, idx, processor)?;
        }
        Value::LineString(ref geometry) => {
            process_linestring(geometry, true, idx, processor)?;
        }
        Value::MultiLineString(ref geometry) => {
            process_multilinestring(geometry, idx, processor)?;
        }
        Value::Polygon(ref geometry) => {
            process_polygon(geometry, true, idx, processor)?;
        }
        Value::MultiPolygon(ref geometry) => {
            process_multi_polygon(geometry, idx, processor)?;
        }
        Value::GeometryCollection(ref collection) => {
            // processor.geomcollection_begin(collection.len());
            for (idx, geometry) in collection.iter().enumerate() {
                match_geometry(geometry, idx, processor)?;
            }
        }
    }
    Ok(())
}

type Position = Vec<f64>;
type PointType = Position;
type LineStringType = Vec<Position>;
type PolygonType = Vec<Vec<Position>>;

fn process_point<P: GeomProcessor>(
    point_type: &PointType,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.point_begin(idx)?;
    processor.xy(point_type[0], point_type[1], 0)?;
    processor.point_end(idx)
}

fn process_multi_point<P: GeomProcessor>(
    multi_point_type: &[PointType],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multipoint_begin(multi_point_type.len(), idx)?;
    for (idxc, point_type) in multi_point_type.iter().enumerate() {
        processor.xy(point_type[0], point_type[1], idxc)?;
    }
    processor.multipoint_end(idx)
}

fn process_linestring<P: GeomProcessor>(
    linestring_type: &LineStringType,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.linestring_begin(tagged, linestring_type.len(), idx)?;
    for (idxc, point_type) in linestring_type.iter().enumerate() {
        processor.xy(point_type[0], point_type[1], idxc)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_multilinestring<P: GeomProcessor>(
    multilinestring_type: &[LineStringType],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multilinestring_begin(multilinestring_type.len(), idx)?;
    for (idxc, linestring_type) in multilinestring_type.iter().enumerate() {
        process_linestring(&linestring_type, false, idxc, processor)?
    }
    processor.multilinestring_end(idx)
}

fn process_polygon<P: GeomProcessor>(
    polygon_type: &PolygonType,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.polygon_begin(tagged, polygon_type.len(), idx)?;
    for (idxl, linestring_type) in polygon_type.iter().enumerate() {
        process_linestring(linestring_type, false, idxl, processor)?
    }
    processor.polygon_end(tagged, idx)
}

fn process_multi_polygon<P: GeomProcessor>(
    multi_polygon_type: &[PolygonType],
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.multipolygon_begin(multi_polygon_type.len(), idx)?;
    for (idxp, polygon_type) in multi_polygon_type.iter().enumerate() {
        process_polygon(&polygon_type, false, idxp, processor)?;
    }
    processor.multipolygon_end(idx)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::wkt_writer::WktWriter;
    use std::fs::File;

    #[test]
    fn line_string() -> Result<()> {
        let geojson = r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#;
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(read_geojson_geom(geojson.as_bytes(), &mut WktWriter::new(&mut wkt_data)).is_ok());
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(wkt, "LINESTRING (1875038.447610231 -3269648.6879248763, 1874359.641504197 -3270196.812984864, 1874141.0428635243 -3270953.7840121365, 1874440.1778162003 -3271619.4315206874, 1876396.0598222911 -3274138.747656357, 1876442.0805243007 -3275052.60551469, 1874739.312657555 -3275457.333765534)"
    );
        Ok(())
    }

    #[test]
    fn feature_collection() -> Result<()> {
        let geojson = r#"{"type": "FeatureCollection", "name": "countries", "features": [{"type": "Feature", "properties": {"id": "NZL", "name": "New Zealand"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}}]}"#;
        let mut wkt_data: Vec<u8> = Vec::new();
        assert!(read_geojson(geojson.as_bytes(), &mut WktWriter::new(&mut wkt_data)).is_ok());
        let wkt = std::str::from_utf8(&wkt_data).unwrap();
        assert_eq!(wkt, "MULTIPOLYGON (((173.020375 -40.919052, 173.247234 -41.331999, 173.958405 -40.926701, 174.247587 -41.349155, 174.248517 -41.770008, 173.876447 -42.233184, 173.22274 -42.970038, 172.711246 -43.372288, 173.080113 -43.853344, 172.308584 -43.865694, 171.452925 -44.242519, 171.185138 -44.897104, 170.616697 -45.908929, 169.831422 -46.355775, 169.332331 -46.641235, 168.411354 -46.619945, 167.763745 -46.290197, 166.676886 -46.219917, 166.509144 -45.852705, 167.046424 -45.110941, 168.303763 -44.123973, 168.949409 -43.935819, 169.667815 -43.555326, 170.52492 -43.031688, 171.12509 -42.512754, 171.569714 -41.767424, 171.948709 -41.514417, 172.097227 -40.956104, 172.79858 -40.493962, 173.020375 -40.919052)), ((174.612009 -36.156397, 175.336616 -37.209098, 175.357596 -36.526194, 175.808887 -36.798942, 175.95849 -37.555382, 176.763195 -37.881253, 177.438813 -37.961248, 178.010354 -37.579825, 178.517094 -37.695373, 178.274731 -38.582813, 177.97046 -39.166343, 177.206993 -39.145776, 176.939981 -39.449736, 177.032946 -39.879943, 176.885824 -40.065978, 176.508017 -40.604808, 176.01244 -41.289624, 175.239567 -41.688308, 175.067898 -41.425895, 174.650973 -41.281821, 175.22763 -40.459236, 174.900157 -39.908933, 173.824047 -39.508854, 173.852262 -39.146602, 174.574802 -38.797683, 174.743474 -38.027808, 174.697017 -37.381129, 174.292028 -36.711092, 174.319004 -36.534824, 173.840997 -36.121981, 173.054171 -35.237125, 172.636005 -34.529107, 173.007042 -34.450662, 173.551298 -35.006183, 174.32939 -35.265496, 174.612009 -36.156397)))");
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
            "POINT (32.533299524864844 0.583299105614628), POINT (30.27500161597942 0.671004121125236), POINT (15"
        );
        assert_eq!(
            &wkt[wkt.len()-100..],
            "0862875), POINT (103.85387481909902 1.294979325105942), POINT (114.18306345846304 22.30692675357551)"
        );
        Ok(())
    }
}
