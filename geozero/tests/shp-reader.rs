use std::fs::File;
use std::io::{BufReader, Cursor};
use std::str::from_utf8;

use dbase::FieldValue;
use geozero::geojson::GeoJsonWriter;
use geozero::shp::ShpReader;
use geozero::wkt::WktWriter;
use geozero::{CoordDimensions, FeatureProperties, ProcessorSink};
use rstest::rstest;

fn shape_record(number: i32, size_16_bit: i32, body: &[u8]) -> Vec<u8> {
    let mut record = number.to_be_bytes().to_vec();
    record.extend_from_slice(&size_16_bit.to_be_bytes());
    record.extend_from_slice(body);
    record
}

fn point_body(x: f64, y: f64) -> Vec<u8> {
    let mut body = (geozero::shp::ShapeType::Point as i32)
        .to_le_bytes()
        .to_vec();
    body.extend_from_slice(&x.to_le_bytes());
    body.extend_from_slice(&y.to_le_bytes());
    body
}

fn shp_header(file_length_16_bit: i32) -> Vec<u8> {
    let mut header = Vec::with_capacity(100);
    header.extend_from_slice(&9994i32.to_be_bytes());
    header.extend_from_slice(&[0; 20]);
    header.extend_from_slice(&file_length_16_bit.to_be_bytes());
    header.extend_from_slice(&1000i32.to_le_bytes());
    header.extend_from_slice(&(geozero::shp::ShapeType::NullShape as i32).to_le_bytes());
    header.extend_from_slice(&[0; 64]);
    assert_eq!(header.len(), 100);
    header
}

fn null_shape_then_point(null_size_16_bit: i32) -> Vec<u8> {
    let null_shape = (geozero::shp::ShapeType::NullShape as i32).to_le_bytes();
    let first = shape_record(1, null_size_16_bit, &null_shape);
    let second = shape_record(2, 10, &point_body(3.0, 4.0));
    let mut shp = shp_header(((100 + first.len() + second.len()) / 2) as i32);
    shp.extend(first);
    shp.extend(second);
    shp
}

#[test]
fn null_shape_record_size_must_match_its_four_byte_body() {
    let mut sink = ProcessorSink::new();
    let reader = ShpReader::new(Cursor::new(null_shape_then_point(2))).unwrap();
    let mut records = reader.iter_geometries(&mut sink);
    assert!(records.next().unwrap().is_ok());
    assert!(records.next().unwrap().is_ok());
    assert!(records.next().is_none());
}

#[rstest]
#[case::just_over(3)]
#[case::double(8)]
#[case::way_over(20)]
#[case::wildly_over(1000)]
fn null_shape_record_with_wrong_size_is_rejected(#[case] size_16_bit: i32) {
    let mut sink = ProcessorSink::new();
    let reader = ShpReader::new(Cursor::new(null_shape_then_point(size_16_bit))).unwrap();
    let mut records = reader.iter_geometries(&mut sink);
    assert!(
        matches!(
            records.next(),
            Some(Err(geozero::shp::Error::InvalidShapeRecordSize))
        ),
        "NullShape record with declared size {size_16_bit} must be rejected"
    );
    assert!(records.next().is_none());
}

#[test]
fn read_header() {
    let reader = ShpReader::from_path("./tests/data/shp/line.shp").unwrap();
    let header = reader.header();
    assert_eq!(header.file_length, 136);
    assert_eq!(header.shape_type, geozero::shp::ShapeType::Polyline);
    assert_eq!(header.bbox.x_range(), [1.0, 5.0]);
}

#[test]
fn iterate() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/poly.shp")?;
    let mut cnt = 0;
    for _ in reader.iter_geometries(&mut ProcessorSink::new()) {
        cnt += 1;
    }
    assert_eq!(cnt, 10);

    let reader = ShpReader::from_path("./tests/data/shp/poly.shp")?;
    let mut cnt = 0;
    for feat in reader.iter_features(&mut ProcessorSink::new())? {
        if let Ok(feat) = feat {
            assert!(feat.property::<f64>("EAS_ID").unwrap() > 100.0);
        }
        cnt += 1;
    }
    assert_eq!(cnt, 10);

    let source = BufReader::new(File::open("./tests/data/shp/poly.shp")?);
    let reader = ShpReader::new(source)?;
    let mut cnt = 0;
    for _ in reader.iter_geometries(&mut ProcessorSink::new()) {
        cnt += 1;
    }
    assert_eq!(cnt, 10);

    Ok(())
}

#[test]
fn shp_to_json() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/poly.shp")?;
    let mut json: Vec<u8> = Vec::new();
    let cnt = reader
        .iter_features(&mut GeoJsonWriter::new(&mut json))?
        .count();
    assert_eq!(cnt, 10);
    assert_eq!(
        &from_utf8(&json).unwrap()[0..80],
        r#"{
"type": "FeatureCollection",
"features": [{"type": "Feature", "properties": {""#
    );
    assert_eq!(
        &from_utf8(&json).unwrap()[json.len() - 100..],
        "2],[479658.59375,4764670],[479640.09375,4764721],[479735.90625,4764752],[479750.6875,4764702]]]]}}]}"
    );
    Ok(())
}

#[test]
fn shp_to_geo() -> Result<(), geozero::shp::Error> {
    use geo_types::Geometry;
    use geozero::geo_types::GeoWriter;

    let reader = ShpReader::from_path("./tests/data/shp/poly.shp")?;
    let mut geo = GeoWriter::new();
    let mut cnt = 0;
    for _geom in reader.iter_geometries(&mut geo) {
        cnt += 1;
    }
    assert_eq!(cnt, 10);
    if let Some(Geometry::GeometryCollection(geo_types::GeometryCollection(gc))) =
        geo.take_geometry()
    {
        assert_eq!(gc.len(), 10);
    } else {
        panic!("unexpected geometry");
    }

    Ok(())
}

#[test]
fn property_filter() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/poly.shp")?;
    let mut json: Vec<u8> = Vec::new();
    let cnt = reader
        .iter_features(&mut GeoJsonWriter::new(&mut json))?
        .filter(|feat| feat.as_ref().unwrap().property::<f64>("AREA").unwrap() > 260000.0)
        .count();
    assert_eq!(cnt, 5);
    // Filter does not work as expected. *All* features are written and converted into GeoJSON!
    assert!(from_utf8(&json).unwrap().contains(r#""AREA": 5268.813"#));
    Ok(())
}

#[test]
fn property_access() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/poly.shp")?;
    let mut cnt = 0;
    for feat in reader.iter_features(&mut ProcessorSink::new())? {
        if let Ok(feat) = feat {
            // Access internal type
            if let Some(FieldValue::Numeric(Some(val))) = feat.record.get("EAS_ID") {
                assert!(*val > 100.0);
            } else {
                panic!("record field access failed");
            }
            // Use String HashMap
            let props = feat.properties()?;
            assert!(props["EAS_ID"].starts_with('1'));
            // field access
            assert!(feat.property::<f64>("EAS_ID").unwrap() > 100.0);
        } else {
            panic!("record field access failed");
        }
        cnt += 1;
    }
    assert_eq!(cnt, 10);

    Ok(())
}

#[test]
fn property_file() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/poly.shp")?;
    let fields = reader.dbf_fields().unwrap();
    assert_eq!(fields.len(), 3);
    let sql = fields
        .iter()
        .map(|f| {
            let name = f.name();
            let _len = f.length();

            let col_type: u8 = f.field_type().into();
            let sql_type = match col_type as char {
                'C' => String::from("TEXT"),
                'D' => String::from("INTEGER"),
                'F' => String::from("REAL"),
                'N' => String::from("REAL"),
                'L' => String::from("INTEGER"),
                'Y' => String::from("REAL"),
                'T' => String::from("INTEGER"),
                'I' => String::from("INTEGER"),
                'B' => String::from("REAL"),
                'M' => String::from("BLOB"),
                _ => unimplemented!(),
            };
            format!("{name} {sql_type}")
        })
        .collect::<Vec<String>>()
        .join(",");
    assert_eq!(sql, "AREA REAL,EAS_ID REAL,PRFEDEA TEXT");
    Ok(())
}

#[test]
fn point() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/point.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(from_utf8(&wkt_data).unwrap(), "POINT(122 37)");
    Ok(())
}

#[test]
fn pointzm() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/pointm.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xym());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "POINT(160477.9000324604 5403959.561417906 0)"
    );

    let reader = ShpReader::from_path("./tests/data/shp/pointz.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyz());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "POINT(1422464.3681007193 4188962.3364355816 72.40956470558095)"
    );
    Ok(())
}

#[test]
fn multipoint() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/multipoint.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(from_utf8(&wkt_data).unwrap(), "MULTIPOINT(122 37,124 32)");
    Ok(())
}

#[test]
fn multipointzm() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/multipointz.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyz());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOINT(1422671.7232666016 4188903.4295959473 72.00995635986328,1422672.1022949219 4188903.4295959473 72.0060806274414,1422671.9127807617 4188903.7578430176 72.00220489501953,1422671.9127807617 4188903.539001465 71.99445343017578)"
    );
    Ok(())
}

#[test]
fn line() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/line.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5,5 5,5 1,3 3,1 1),(3 2,2 6))"
    );
    Ok(())
}

#[test]
fn linezm() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/linez.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyzm());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5 18 -1000000000000000000000000000000000000000,5 5 20 -1000000000000000000000000000000000000000,5 1 22 -1000000000000000000000000000000000000000,3 3 0 -1000000000000000000000000000000000000000,1 1 0 -1000000000000000000000000000000000000000),(3 2 0 -1000000000000000000000000000000000000000,2 6 0 -1000000000000000000000000000000000000000),(3 2 15 0,2 6 13 3,1 9 14 2))"
    );

    let reader = ShpReader::from_path("./tests/data/shp/linez.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyz());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5 18,5 5 20,5 1 22,3 3 0,1 1 0),(3 2 0,2 6 0),(3 2 15,2 6 13,1 9 14))"
    );

    let reader = ShpReader::from_path("./tests/data/shp/linez.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::new(&mut wkt_data);
    // return XY only
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5,5 5,5 1,3 3,1 1),(3 2,2 6),(3 2,2 6,1 9))"
    );

    let reader = ShpReader::from_path("./tests/data/shp/linem.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xym());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5 0,5 5 -1000000000000000000000000000000000000000,5 1 3,3 3 -1000000000000000000000000000000000000000,1 1 0),(3 2 -1000000000000000000000000000000000000000,2 6 -1000000000000000000000000000000000000000))"
    );

    Ok(())
}

#[test]
fn polygon() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/polygon.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((122 37,117 36,115 32,118 20,113 24)),((15 2,17 6,22 7),(122 37,117 36,115 32)))" //ogrinfo: "MULTIPOLYGON(((122 37,117 36,115 32,118 20,113 24)),((15 2,17 6,22 7)),((122 37,117 36,115 32)))"
    );

    let reader = ShpReader::from_path("./tests/data/shp/polygon_hole.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((-120 60,120 60,120 -60,-120 -60,-120 60),(-60 30,-60 -30,60 -30,60 30,-60 30)))"
    );

    let reader = ShpReader::from_path("./tests/data/shp/multi_polygon.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(
        &from_utf8(&wkt_data).unwrap()[0..100],
        "MULTIPOLYGON(((5.879502799999998 43.13421680053936,5.8798122999999975 43.13437570053936,5.8801381999"
    );
    assert_eq!(
        &from_utf8(&wkt_data).unwrap()[wkt_data.len() - 1067..],
        "5.923433499999997 43.11938760053909)),((5.9547390999999985 43.10615080053885,5.9548353999999994 43.106223500538846,5.954922299999998 43.10636420053886,5.954951999999997 43.106424900538855,5.955154899999998 43.10636740053886,5.955408999999999 43.106533300538864,5.955599199999998 43.10659070053885,5.955937999999998 43.10670310053886,5.955992099999998 43.106726200538866,5.956030399999998 43.10675580053888,5.956104699999998 43.10684620053886,5.956232599999999 43.10701230053886,5.956314199999998 43.107038600538864,5.9563704999999985 43.10701060053888,5.956408799999998 43.106963000538876,5.956242099999998 43.10679360053885,5.956138499999997 43.10667190053885,5.956356999999999 43.106351200538846,5.956746599999998 43.106375900538865,5.956832199999998 43.10628380053884,5.956746599999998 43.10621640053884,5.956269799999999 43.106223500538846,5.956027699999999 43.106182700538845,5.9557854999999975 43.106073900538846,5.955412899999998 43.106005900538854,5.955170699999998 43.10601950053885,5.954942199999998 43.10605660053885,5.9547390999999985 43.10615080053885)))"
    );
    Ok(())
}

#[test]
fn polygonzm() -> Result<(), geozero::shp::Error> {
    let reader = ShpReader::from_path("./tests/data/shp/polygonz.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyzm());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((1422692.1644789441 4188837.794210903 72.46632654472523 0,1422692.1625749937 4188837.75060327 72.46632654472523 1,1422692.156877633 4188837.7073275167 72.46632654472523 2,1422692.1474302218 4188837.664712999 72.46632654472523 3,1422692.1343046608 4188837.6230840385 72.46632654472523 4,1422692.1176008438 4188837.582757457 72.46632654472523 5,1422692.0974458966 4188837.5440401635 72.46632654472523 6,1422692.0739932107 4188837.5072268206 72.46632654472523 7,1422692.047421275 4188837.4725976 72.46632654472523 8,1422692.017932318 4188837.4404160506 72.46632654472523 9,1422691.9857507686 4188837.4109270936 72.46632654472523 10,1422691.951121548 4188837.384355158 72.46632654472523 11,1422691.914308205 4188837.360902472 72.46632654472523 12,1422691.8755909116 4188837.3407475245 72.46632654472523 13,1422691.8352643298 4188837.3240437075 72.46632654472523 14,1422691.7936353693 4188837.3109181467 72.46632654472523 15,1422691.7510208515 4188837.3014707356 72.46632654472523 16,1422691.7077450987 4188837.295773375 72.46632654472523 17,1422691.6641374656 4188837.293869424 72.46632654472523 18,1422691.6205298326 4188837.295773375 72.46632654472523 19,1422691.5772540797 4188837.3014707356 72.46632654472523 20,1422691.534639562 4188837.3109181467 72.46632654472523 21,1422691.4930106015 4188837.3240437075 72.46632654472523 22,1422691.4526840197 4188837.3407475245 72.46632654472523 23,1422691.4139667263 4188837.360902472 72.46632654472523 24,1422691.3771533833 4188837.384355158 72.46632654472523 25,1422691.3425241627 4188837.4109270936 72.46632654472523 26,1422691.3103426134 4188837.4404160506 72.46632654472523 27,1422691.2808536564 4188837.4725976 72.46632654472523 28,1422691.2542817206 4188837.5072268206 72.46632654472523 29,1422691.2308290347 4188837.5440401635 72.46632654472523 30,1422691.2106740875 4188837.582757457 72.46632654472523 31,1422691.1939702705 4188837.6230840385 72.46632654472523 32,1422691.1808447095 4188837.664712999 72.46632654472523 33,1422691.1713972983 4188837.7073275167 72.46632654472523 34,1422691.1656999376 4188837.75060327 72.46632654472523 35,1422691.1637959871 4188837.794210903 72.46632654472523 36,1422691.1656999376 4188837.837818536 72.46632654472523 37,1422691.1713972983 4188837.881094289 72.46632654472523 38,1422691.1808447095 4188837.9237088067 72.46632654472523 39,1422691.1939702705 4188837.9653377673 72.46632654472523 40,1422691.2106740875 4188838.0056643486 72.46632654472523 41,1422691.2308290347 4188838.0443816422 72.46632654472523 42,1422691.2542817206 4188838.081194985 72.46632654472523 43,1422691.2808536564 4188838.115824206 72.46632654472523 44,1422691.3103426134 4188838.148005755 72.46632654472523 45,1422691.3425241627 4188838.177494712 72.46632654472523 46,1422691.3771533833 4188838.2040666477 72.46632654472523 47,1422691.4139667263 4188838.227519334 72.46632654472523 48,1422691.4526840197 4188838.2476742812 72.46632654472523 49,1422691.4930106015 4188838.2643780983 72.46632654472523 50,1422691.534639562 4188838.277503659 72.46632654472523 51,1422691.5772540797 4188838.28695107 72.46632654472523 52,1422691.6205298326 4188838.292648431 72.46632654472523 53,1422691.6641374656 4188838.2945523816 72.46632654472523 54,1422691.7077450987 4188838.292648431 72.46632654472523 55,1422691.7510208515 4188838.28695107 72.46632654472523 56,1422691.7936353693 4188838.277503659 72.46632654472523 57,1422691.8352643298 4188838.2643780983 72.46632654472523 58,1422691.8755909116 4188838.2476742812 72.46632654472523 59,1422691.914308205 4188838.227519334 72.46632654472523 60,1422691.951121548 4188838.2040666477 72.46632654472523 61,1422691.9857507686 4188838.177494712 72.46632654472523 62,1422692.017932318 4188838.148005755 72.46632654472523 63,1422692.047421275 4188838.115824206 72.46632654472523 64,1422692.0739932107 4188838.081194985 72.46632654472523 65,1422692.0974458966 4188838.0443816422 72.46632654472523 66,1422692.1176008438 4188838.0056643486 72.46632654472523 67,1422692.1343046608 4188837.9653377673 72.46632654472523 68,1422692.1474302218 4188837.9237088067 72.46632654472523 69,1422692.156877633 4188837.881094289 72.46632654472523 70,1422692.1625749937 4188837.837818536 72.46632654472523 71,1422692.1644789441 4188837.794210903 72.46632654472523 72)))"
    );

    let reader = ShpReader::from_path("./tests/data/shp/polygonm.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xym());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((159814.75390576152 5404314.139043656 0,160420.36722814097 5403703.520652497 0,159374.30785312195 5403473.287488617 0,159814.75390576152 5404314.139043656 0)))"
    );

    Ok(())
}

// --- untrusted-count preallocation / overflow DoS regression tests ---
//
// Mirrors the WKB hardening (#297/#299): every `.shp`/`.shx` count field decoded
// off untrusted bytes must be validated before being handed to a `Vec` reservation,
// so a tiny malformed record returns `Err` instead of panicking (`capacity overflow`
// / debug int-overflow) or requesting a multi-GB allocation.

/// Build a valid 100-byte shapefile main header so `ShpReader::new` accepts the
/// stream; `file_length_words` is the (attacker-controlled) file-length field.
fn shp_main_header(file_length_words: i32) -> Vec<u8> {
    let mut h = Vec::with_capacity(100);
    h.extend_from_slice(&9994i32.to_be_bytes()); // file_code
    h.extend_from_slice(&[0u8; 20]); // 5x i32 skip
    h.extend_from_slice(&file_length_words.to_be_bytes()); // file_length (16-bit words), BE
    h.extend_from_slice(&1000i32.to_le_bytes()); // version, LE
    h.extend_from_slice(&0i32.to_le_bytes()); // shape_type (NullShape), LE
    h.extend_from_slice(&[0u8; 64]); // 8x f64 bbox
    assert_eq!(h.len(), 100);
    h
}

/// Drive the public `ShpReader::new(Cursor) -> iter_geometries` path with `bytes`
/// and collect the first result. Returns `Ok(())` only if the iterator yields an
/// `Err` (the safe outcome) rather than panicking.
fn assert_record_errors(name: &str, bytes: Vec<u8>) {
    let result = std::panic::catch_unwind(|| {
        let reader = ShpReader::new(Cursor::new(bytes)).expect("header parse");
        let mut sink = ProcessorSink::new();
        reader.iter_geometries(&mut sink).next()
    });
    match result {
        Ok(None) => panic!("[{name}] iterator yielded None (no record read)"),
        Ok(Some(Ok(_))) => panic!("[{name}] record decoded successfully from malformed input"),
        Ok(Some(Err(_))) => { /* the safe outcome: malformed record rejected */ }
        Err(payload) => {
            let msg = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&'static str>().copied())
                .unwrap_or("<non-string panic>");
            panic!("[{name}] PANIC on malformed input (regression): {msg}");
        }
    }
}

#[test]
fn multipatch_negative_record_size_is_rejected_not_panicked() {
    // record_size (BE i32) = -1 -> vec![0; ~1.8e19] capacity overflow before the fix.
    let mut b = shp_main_header(500);
    b.extend_from_slice(&1i32.to_be_bytes()); // record_number
    b.extend_from_slice(&(-1i32).to_be_bytes()); // record_size (untrusted) = -1
    b.extend_from_slice(&31i32.to_le_bytes()); // shape_type = Multipatch
    assert_record_errors("multipatch record_size=-1", b);
}

#[test]
fn multipatch_huge_record_size_is_rejected_not_oom() {
    // record_size (BE i32) = i32::MAX -> after *2 a multi-GB reservation.
    let mut b = shp_main_header(500);
    b.extend_from_slice(&1i32.to_be_bytes());
    b.extend_from_slice(&i32::MAX.to_be_bytes()); // record_size = i32::MAX (untrusted)
    b.extend_from_slice(&31i32.to_le_bytes()); // shape_type = Multipatch
    assert_record_errors("multipatch record_size=MAX", b);
}

#[test]
fn record_size_below_shape_type_is_rejected_not_panicked() {
    // record_size = 0 is non-negative but smaller than the 4-byte shape type it
    // contains, so `record_size - size_of::<i32>()` underflowed (debug panic /
    // release wrap to ~1.8e19).
    let mut b = shp_main_header(500);
    b.extend_from_slice(&1i32.to_be_bytes()); // record_number
    b.extend_from_slice(&0i32.to_be_bytes()); // record_size = 0 (untrusted)
    b.extend_from_slice(&1i32.to_le_bytes()); // shape_type = Point
    assert_record_errors("record_size=0", b);
}

#[test]
fn polygon_negative_num_points_is_rejected_not_panicked() {
    // num_points (LE i32) = -1 -> multipart_record_size(16 * -1) debug-mul-overflow /
    // read_xy Vec::with_capacity(~1.8e19) capacity overflow before the fix.
    let mut b = shp_main_header(500);
    b.extend_from_slice(&1i32.to_be_bytes()); // record_number
    b.extend_from_slice(&1000i32.to_be_bytes()); // record_size (large enough to survive the check)
    b.extend_from_slice(&5i32.to_le_bytes()); // shape_type = Polygon
    b.extend_from_slice(&[0u8; 32]); // bbox (4x f64)
    b.extend_from_slice(&1i32.to_le_bytes()); // num_parts = 1
    b.extend_from_slice(&(-1i32).to_le_bytes()); // num_points = -1 (untrusted)
    assert_record_errors("polygon num_points=-1", b);
}

#[test]
fn multipoint_negative_num_points_is_rejected_not_panicked() {
    // Multipoint with num_points (LE i32) = -1.
    let mut b = shp_main_header(500);
    b.extend_from_slice(&1i32.to_be_bytes()); // record_number
    b.extend_from_slice(&1000i32.to_be_bytes()); // record_size
    b.extend_from_slice(&8i32.to_le_bytes()); // shape_type = Multipoint
    b.extend_from_slice(&[0u8; 32]); // bbox (4x f64)
    b.extend_from_slice(&(-1i32).to_le_bytes()); // num_points = -1 (untrusted)
    assert_record_errors("multipoint num_points=-1", b);
}

#[test]
fn polyline_huge_num_parts_is_rejected_not_oom() {
    // num_parts (LE i32) = i32::MAX -> Vec::with_capacity(num_parts + 1) huge reservation.
    let mut b = shp_main_header(500);
    b.extend_from_slice(&1i32.to_be_bytes()); // record_number
    b.extend_from_slice(&i32::MAX.to_be_bytes()); // record_size (untrusted, large)
    b.extend_from_slice(&3i32.to_le_bytes()); // shape_type = Polyline
    b.extend_from_slice(&[0u8; 32]); // bbox (4x f64)
    b.extend_from_slice(&i32::MAX.to_le_bytes()); // num_parts = i32::MAX (untrusted)
    b.extend_from_slice(&0i32.to_le_bytes()); // num_points = 0
    assert_record_errors("polyline num_parts=MAX", b);
}

#[test]
fn shx_negative_file_length_is_rejected_not_panicked() {
    // A .shx whose file_length (BE i32) is negative sign-extends under `as usize`
    // to ~1.8e19 and the ShapeIndex Vec reservation overflows capacity / OOMs.
    let mut reader = ShpReader::new(Cursor::new(shp_main_header(500))).expect("shp header parse");
    // Returns Err (not a panic / OOM) after the fix.
    let res = reader.add_index_source(Cursor::new(shp_main_header(-1)));
    assert!(
        res.is_err(),
        "add_index_source accepted negative file_length"
    );
}

#[test]
fn shx_huge_file_length_is_rejected_not_oom() {
    // file_length = i32::MAX -> after *2 and /INDEX_RECORD_SIZE a huge reservation.
    let mut reader = ShpReader::new(Cursor::new(shp_main_header(500))).expect("shp header parse");
    let res = reader.add_index_source(Cursor::new(shp_main_header(i32::MAX)));
    assert!(res.is_err(), "add_index_source accepted huge file_length");
}

#[test]
fn legit_shapefiles_still_parse() {
    // Regression guard: legitimate fixtures must keep decoding after the budget.
    for f in &[
        "line.shp",
        "poly.shp",
        "point.shp",
        "pointm.shp",
        "pointz.shp",
        "multipoint.shp",
        "multipointz.shp",
        "polygon.shp",
    ] {
        let reader = ShpReader::from_path(format!("./tests/data/shp/{f}")).expect("open fixture");
        let mut sink = ProcessorSink::new();
        let mut n = 0;
        for res in reader.iter_geometries(&mut sink) {
            assert!(res.is_ok(), "legit fixture {f} record failed: {res:?}");
            n += 1;
        }
        assert!(n > 0, "legit fixture {f} yielded no records");
    }
}
