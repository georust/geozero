use dbase::FieldValue;
use geozero::geojson::GeoJsonWriter;
use geozero::wkt::WktWriter;
use geozero::{CoordDimensions, FeatureProperties, ProcessorSink};
use std::fs::File;
use std::io::BufReader;
use std::str::from_utf8;

#[test]
fn read_header() {
    let reader = geozero_shp::Reader::from_path("./tests/data/line.shp").unwrap();
    let header = reader.header();
    assert_eq!(header.file_length, 136);
    assert_eq!(header.shape_type, geozero_shp::ShapeType::Polyline);
    assert_eq!(header.bbox.x_range(), [1.0, 5.0]);
}

#[test]
fn iterate() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/poly.shp")?;
    let mut cnt = 0;
    for _ in reader.iter_geometries(&mut ProcessorSink::new()) {
        cnt += 1;
    }
    assert_eq!(cnt, 10);

    let reader = geozero_shp::Reader::from_path("./tests/data/poly.shp")?;
    let mut cnt = 0;
    for feat in reader.iter_features(&mut ProcessorSink::new())? {
        if let Ok(feat) = feat {
            assert!(feat.property::<f64>("EAS_ID").unwrap() > 100.0);
        }
        cnt += 1;
    }
    assert_eq!(cnt, 10);

    let source = BufReader::new(File::open("./tests/data/poly.shp")?);
    let reader = geozero_shp::Reader::new(source)?;
    let mut cnt = 0;
    for _ in reader.iter_geometries(&mut ProcessorSink::new()) {
        cnt += 1;
    }
    assert_eq!(cnt, 10);

    Ok(())
}

#[test]
fn shp_to_json() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/poly.shp")?;
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
        &from_utf8(&json).unwrap()[json.len()-100..],
        "2],[479658.59375,4764670],[479640.09375,4764721],[479735.90625,4764752],[479750.6875,4764702]]]]}}]}"
    );
    Ok(())
}

#[test]
fn shp_to_geo() -> Result<(), geozero_shp::Error> {
    use geo_types::Geometry;
    use geozero::geo_types::GeoWriter;

    let reader = geozero_shp::Reader::from_path("./tests/data/poly.shp")?;
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
fn property_filter() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/poly.shp")?;
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
fn property_access() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/poly.shp")?;
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
fn property_file() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/poly.shp")?;
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
fn point() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/point.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(from_utf8(&wkt_data).unwrap(), "POINT(122 37)");
    Ok(())
}

#[test]
fn pointzm() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/pointm.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xym());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "POINT(160477.9000324604 5403959.561417906 0)"
    );

    let reader = geozero_shp::Reader::from_path("./tests/data/pointz.shp")?;
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
fn multipoint() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/multipoint.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(from_utf8(&wkt_data).unwrap(), "MULTIPOINT(122 37,124 32)");
    Ok(())
}

#[test]
fn multipointzm() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/multipointz.shp")?;
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
fn line() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/line.shp")?;
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
fn linezm() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/linez.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyzm());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5 18 -1000000000000000000000000000000000000000,5 5 20 -1000000000000000000000000000000000000000,5 1 22 -1000000000000000000000000000000000000000,3 3 0 -1000000000000000000000000000000000000000,1 1 0 -1000000000000000000000000000000000000000),(3 2 0 -1000000000000000000000000000000000000000,2 6 0 -1000000000000000000000000000000000000000),(3 2 15 0,2 6 13 3,1 9 14 2))"
    );

    let reader = geozero_shp::Reader::from_path("./tests/data/linez.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyz());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5 18,5 5 20,5 1 22,3 3 0,1 1 0),(3 2 0,2 6 0),(3 2 15,2 6 13,1 9 14))"
    );

    let reader = geozero_shp::Reader::from_path("./tests/data/linez.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::new(&mut wkt_data);
    // return XY only
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTILINESTRING((1 5,5 5,5 1,3 3,1 1),(3 2,2 6),(3 2,2 6,1 9))"
    );

    let reader = geozero_shp::Reader::from_path("./tests/data/linem.shp")?;
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
fn polygon() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/polygon.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((122 37,117 36,115 32,118 20,113 24)),((15 2,17 6,22 7),(122 37,117 36,115 32)))"
        //ogrinfo: "MULTIPOLYGON(((122 37,117 36,115 32,118 20,113 24)),((15 2,17 6,22 7)),((122 37,117 36,115 32)))"
    );

    let reader = geozero_shp::Reader::from_path("./tests/data/polygon_hole.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((-120 60,120 60,120 -60,-120 -60,-120 60),(-60 30,-60 -30,60 -30,60 30,-60 30)))"
    );

    let reader = geozero_shp::Reader::from_path("./tests/data/multi_polygon.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    reader
        .iter_geometries(&mut WktWriter::new(&mut wkt_data))
        .next();
    assert_eq!(
        &from_utf8(&wkt_data).unwrap()[0..100],
        "MULTIPOLYGON(((5.879502799999998 43.13421680053936,5.8798122999999975 43.13437570053936,5.8801381999"
    );
    assert_eq!(
        &from_utf8(&wkt_data).unwrap()[wkt_data.len()-1067..],
        "5.923433499999997 43.11938760053909)),((5.9547390999999985 43.10615080053885,5.9548353999999994 43.106223500538846,5.954922299999998 43.10636420053886,5.954951999999997 43.106424900538855,5.955154899999998 43.10636740053886,5.955408999999999 43.106533300538864,5.955599199999998 43.10659070053885,5.955937999999998 43.10670310053886,5.955992099999998 43.106726200538866,5.956030399999998 43.10675580053888,5.956104699999998 43.10684620053886,5.956232599999999 43.10701230053886,5.956314199999998 43.107038600538864,5.9563704999999985 43.10701060053888,5.956408799999998 43.106963000538876,5.956242099999998 43.10679360053885,5.956138499999997 43.10667190053885,5.956356999999999 43.106351200538846,5.956746599999998 43.106375900538865,5.956832199999998 43.10628380053884,5.956746599999998 43.10621640053884,5.956269799999999 43.106223500538846,5.956027699999999 43.106182700538845,5.9557854999999975 43.106073900538846,5.955412899999998 43.106005900538854,5.955170699999998 43.10601950053885,5.954942199999998 43.10605660053885,5.9547390999999985 43.10615080053885)))"
    );
    Ok(())
}

#[test]
fn polygonzm() -> Result<(), geozero_shp::Error> {
    let reader = geozero_shp::Reader::from_path("./tests/data/polygonz.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xyzm());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((1422692.1644789441 4188837.794210903 72.46632654472523 0,1422692.1625749937 4188837.75060327 72.46632654472523 1,1422692.156877633 4188837.7073275167 72.46632654472523 2,1422692.1474302218 4188837.664712999 72.46632654472523 3,1422692.1343046608 4188837.6230840385 72.46632654472523 4,1422692.1176008438 4188837.582757457 72.46632654472523 5,1422692.0974458966 4188837.5440401635 72.46632654472523 6,1422692.0739932107 4188837.5072268206 72.46632654472523 7,1422692.047421275 4188837.4725976 72.46632654472523 8,1422692.017932318 4188837.4404160506 72.46632654472523 9,1422691.9857507686 4188837.4109270936 72.46632654472523 10,1422691.951121548 4188837.384355158 72.46632654472523 11,1422691.914308205 4188837.360902472 72.46632654472523 12,1422691.8755909116 4188837.3407475245 72.46632654472523 13,1422691.8352643298 4188837.3240437075 72.46632654472523 14,1422691.7936353693 4188837.3109181467 72.46632654472523 15,1422691.7510208515 4188837.3014707356 72.46632654472523 16,1422691.7077450987 4188837.295773375 72.46632654472523 17,1422691.6641374656 4188837.293869424 72.46632654472523 18,1422691.6205298326 4188837.295773375 72.46632654472523 19,1422691.5772540797 4188837.3014707356 72.46632654472523 20,1422691.534639562 4188837.3109181467 72.46632654472523 21,1422691.4930106015 4188837.3240437075 72.46632654472523 22,1422691.4526840197 4188837.3407475245 72.46632654472523 23,1422691.4139667263 4188837.360902472 72.46632654472523 24,1422691.3771533833 4188837.384355158 72.46632654472523 25,1422691.3425241627 4188837.4109270936 72.46632654472523 26,1422691.3103426134 4188837.4404160506 72.46632654472523 27,1422691.2808536564 4188837.4725976 72.46632654472523 28,1422691.2542817206 4188837.5072268206 72.46632654472523 29,1422691.2308290347 4188837.5440401635 72.46632654472523 30,1422691.2106740875 4188837.582757457 72.46632654472523 31,1422691.1939702705 4188837.6230840385 72.46632654472523 32,1422691.1808447095 4188837.664712999 72.46632654472523 33,1422691.1713972983 4188837.7073275167 72.46632654472523 34,1422691.1656999376 4188837.75060327 72.46632654472523 35,1422691.1637959871 4188837.794210903 72.46632654472523 36,1422691.1656999376 4188837.837818536 72.46632654472523 37,1422691.1713972983 4188837.881094289 72.46632654472523 38,1422691.1808447095 4188837.9237088067 72.46632654472523 39,1422691.1939702705 4188837.9653377673 72.46632654472523 40,1422691.2106740875 4188838.0056643486 72.46632654472523 41,1422691.2308290347 4188838.0443816422 72.46632654472523 42,1422691.2542817206 4188838.081194985 72.46632654472523 43,1422691.2808536564 4188838.115824206 72.46632654472523 44,1422691.3103426134 4188838.148005755 72.46632654472523 45,1422691.3425241627 4188838.177494712 72.46632654472523 46,1422691.3771533833 4188838.2040666477 72.46632654472523 47,1422691.4139667263 4188838.227519334 72.46632654472523 48,1422691.4526840197 4188838.2476742812 72.46632654472523 49,1422691.4930106015 4188838.2643780983 72.46632654472523 50,1422691.534639562 4188838.277503659 72.46632654472523 51,1422691.5772540797 4188838.28695107 72.46632654472523 52,1422691.6205298326 4188838.292648431 72.46632654472523 53,1422691.6641374656 4188838.2945523816 72.46632654472523 54,1422691.7077450987 4188838.292648431 72.46632654472523 55,1422691.7510208515 4188838.28695107 72.46632654472523 56,1422691.7936353693 4188838.277503659 72.46632654472523 57,1422691.8352643298 4188838.2643780983 72.46632654472523 58,1422691.8755909116 4188838.2476742812 72.46632654472523 59,1422691.914308205 4188838.227519334 72.46632654472523 60,1422691.951121548 4188838.2040666477 72.46632654472523 61,1422691.9857507686 4188838.177494712 72.46632654472523 62,1422692.017932318 4188838.148005755 72.46632654472523 63,1422692.047421275 4188838.115824206 72.46632654472523 64,1422692.0739932107 4188838.081194985 72.46632654472523 65,1422692.0974458966 4188838.0443816422 72.46632654472523 66,1422692.1176008438 4188838.0056643486 72.46632654472523 67,1422692.1343046608 4188837.9653377673 72.46632654472523 68,1422692.1474302218 4188837.9237088067 72.46632654472523 69,1422692.156877633 4188837.881094289 72.46632654472523 70,1422692.1625749937 4188837.837818536 72.46632654472523 71,1422692.1644789441 4188837.794210903 72.46632654472523 72)))"
    );

    let reader = geozero_shp::Reader::from_path("./tests/data/polygonm.shp")?;
    let mut wkt_data: Vec<u8> = Vec::new();
    let mut writer = WktWriter::with_dims(&mut wkt_data, CoordDimensions::xym());
    reader.iter_geometries(&mut writer).next();
    assert_eq!(
        from_utf8(&wkt_data).unwrap(),
        "MULTIPOLYGON(((159814.75390576152 5404314.139043656 0,160420.36722814097 5403703.520652497 0,159374.30785312195 5403473.287488617 0,159814.75390576152 5404314.139043656 0)))"
    );

    Ok(())
}
