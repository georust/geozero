#![cfg(feature = "with-csv")]

use std::io::{self, Write};

use geozero::csv::CsvWriter;
use geozero::error::{GeozeroError, Result};
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};

fn feature<W: Write>(
    writer: &mut CsvWriter<W>,
    index: u64,
    properties: &[(usize, &str, &str)],
) -> Result<()> {
    writer.feature_begin(index)?;
    writer.properties_begin()?;
    for (i, name, value) in properties {
        writer.property(*i, name, &ColumnValue::String(value))?;
    }
    writer.properties_end()?;
    writer.geometry_begin()?;
    writer.point_begin(0)?;
    writer.xy(f64::from(u32::try_from(index).unwrap()), 45.0, 0)?;
    writer.point_end(0)?;
    writer.geometry_end()?;
    writer.feature_end(index)
}

fn records(bytes: &[u8]) -> (Vec<String>, Vec<Vec<String>>) {
    let mut reader = csv::Reader::from_reader(bytes);
    let headers = reader
        .headers()
        .unwrap()
        .iter()
        .map(str::to_owned)
        .collect();
    let rows = reader
        .records()
        .map(|row| row.unwrap().iter().map(str::to_owned).collect())
        .collect();
    (headers, rows)
}

#[test]
fn property_order_follows_first_feature_names() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        feature(&mut writer, 0, &[(0, "a", "first-a"), (1, "b", "first-b")]).unwrap();
        feature(
            &mut writer,
            1,
            &[(0, "b", "second-b"), (1, "a", "second-a")],
        )
        .unwrap();
    }
    let (headers, rows) = records(&bytes);
    assert_eq!(headers, ["geometry", "a", "b"]);
    assert_eq!(rows[0], ["POINT(0 45)", "first-a", "first-b"]);
    assert_eq!(rows[1], ["POINT(1 45)", "second-a", "second-b"]);
}

#[test]
fn callback_indices_do_not_define_csv_columns() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        feature(&mut writer, 0, &[(1, "name", "first")]).unwrap();
        feature(&mut writer, 1, &[(7, "name", "second")]).unwrap();
    }
    let (headers, rows) = records(&bytes);
    assert_eq!(headers, ["geometry", "name"]);
    assert_eq!(rows[1], ["POINT(1 45)", "second"]);
}

#[test]
fn missing_properties_get_empty_cells() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        feature(&mut writer, 0, &[(0, "a", "one"), (1, "b", "two")]).unwrap();
        feature(&mut writer, 1, &[(0, "b", "three")]).unwrap();
        feature(&mut writer, 2, &[]).unwrap();
    }
    let (_, rows) = records(&bytes);
    assert_eq!(rows[1], ["POINT(1 45)", "", "three"]);
    assert_eq!(rows[2], ["POINT(2 45)", "", ""]);
}

#[test]
fn unexpected_property_is_an_error_without_a_partial_row() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        feature(&mut writer, 0, &[(0, "a", "first")]).unwrap();
        let error = feature(&mut writer, 1, &[(0, "new", "second")]).unwrap_err();
        assert!(matches!(error, GeozeroError::Property(_)));
    }
    let (headers, rows) = records(&bytes);
    assert_eq!(headers, ["geometry", "a"]);
    assert_eq!(rows, [vec!["POINT(0 45)", "first"]]);
}

#[test]
fn duplicate_property_names_are_errors_even_for_empty_values() {
    for later_row in [false, true] {
        let mut bytes = Vec::new();
        let mut writer = CsvWriter::new(&mut bytes);
        if later_row {
            feature(&mut writer, 0, &[(0, "name", "initial")]).unwrap();
        }
        writer.feature_begin(1).unwrap();
        writer.properties_begin().unwrap();
        writer
            .property(0, "name", &ColumnValue::String(""))
            .unwrap();
        let error = writer
            .property(1, "name", &ColumnValue::String("again"))
            .unwrap_err();
        assert!(matches!(error, GeozeroError::Property(_)));
    }
}

#[test]
fn first_empty_feature_establishes_geometry_only_schema() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        feature(&mut writer, 0, &[]).unwrap();
        let error = feature(&mut writer, 1, &[(0, "new", "value")]).unwrap_err();
        assert!(matches!(error, GeozeroError::Property(_)));
    }
    let (headers, rows) = records(&bytes);
    assert_eq!(headers, ["geometry"]);
    assert_eq!(rows, [vec!["POINT(0 45)"]]);
}

#[test]
fn empty_collection_writes_no_header() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        writer.dataset_begin(None).unwrap();
        writer.dataset_end().unwrap();
    }
    assert_eq!(bytes, Vec::<u8>::new());
}

#[test]
fn geometry_property_stays_separate_from_geometry_column() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        feature(&mut writer, 0, &[(0, "geometry", "attribute")]).unwrap();
        feature(&mut writer, 1, &[(0, "geometry", "next")]).unwrap();
    }
    let (headers, rows) = records(&bytes);
    assert_eq!(headers, ["geometry", "geometry"]);
    assert_eq!(rows[1], ["POINT(1 45)", "next"]);
}

#[test]
fn property_types_can_change_without_changing_the_column() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        writer.feature_begin(0).unwrap();
        writer.property(0, "value", &ColumnValue::Int(42)).unwrap();
        writer.feature_end(0).unwrap();
        writer.feature_begin(1).unwrap();
        writer
            .property(0, "value", &ColumnValue::String("now"))
            .unwrap();
        writer.feature_end(1).unwrap();
    }
    let (headers, rows) = records(&bytes);
    assert_eq!(headers, ["geometry", "value"]);
    assert_eq!(rows, [vec!["", "42"], vec!["", "now"]]);
}

#[test]
fn names_and_values_keep_csv_escaping() {
    let mut bytes = Vec::new();
    {
        let mut writer = CsvWriter::new(&mut bytes);
        feature(
            &mut writer,
            0,
            &[(0, "comma,name", "one,\"quoted\"\nline"), (1, "b", "plain")],
        )
        .unwrap();
        feature(
            &mut writer,
            1,
            &[(0, "b", "next"), (1, "comma,name", "two\nlines")],
        )
        .unwrap();
    }
    let (headers, rows) = records(&bytes);
    assert_eq!(headers, ["geometry", "comma,name", "b"]);
    assert_eq!(rows[0][1], "one,\"quoted\"\nline");
    assert_eq!(rows[1], ["POINT(1 45)", "two\nlines", "next"]);
}

#[test]
fn writer_errors_propagate() {
    struct FailingWriter;
    impl Write for FailingWriter {
        fn write(&mut self, _bytes: &[u8]) -> io::Result<usize> {
            Err(io::Error::other("ordinary writer failure"))
        }
        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }
    let mut writer = CsvWriter::new(FailingWriter);
    let long = "value".repeat(2000);
    assert!(feature(&mut writer, 0, &[(0, "name", &long)]).is_err());
}

#[cfg(feature = "with-geojson")]
#[test]
fn geojson_null_and_missing_properties_keep_header_alignment() {
    use geozero::ProcessToCsv;
    use geozero::geojson::GeoJson;

    let json = r#"{"type":"FeatureCollection","features":[
        {"type":"Feature","geometry":{"type":"Point","coordinates":[1,2]},"properties":{"a":null,"b":"first"}},
        {"type":"Feature","geometry":{"type":"Point","coordinates":[3,4]},"properties":{"a":null,"b":"second"}},
        {"type":"Feature","geometry":null,"properties":null}
    ]}"#;
    let csv = GeoJson(json).to_csv().unwrap();
    let (headers, rows) = records(csv.as_bytes());
    assert_eq!(headers, ["geometry", "b"]);
    assert_eq!(rows[0], ["POINT(1 2)", "first"]);
    assert_eq!(rows[1], ["POINT(3 4)", "second"]);
    assert_eq!(rows[2], ["", ""]);
}
