#![cfg(feature = "with-geojson")]

use geozero::error::{GeozeroError, Result};
use geozero::geojson::GeoJsonWriter;
use geozero::{ColumnValue, FeatureProcessor, GeomProcessor, PropertyProcessor};
use serde_json::{Value, json};
use std::io::{self, Write};

fn feature(properties: &[(&str, ColumnValue<'_>)]) -> Result<Vec<u8>> {
    let mut out = Vec::new();
    let mut writer = GeoJsonWriter::new(&mut out);
    writer.feature_begin(0)?;
    writer.properties_begin()?;
    for (i, (name, value)) in properties.iter().enumerate() {
        writer.property(i, name, value)?;
    }
    writer.properties_end()?;
    writer.geometry_begin()?;
    writer.point_begin(0)?;
    writer.xy(1.0, 2.0, 0)?;
    writer.point_end(0)?;
    writer.geometry_end()?;
    writer.feature_end(0)?;
    Ok(out)
}

#[test]
fn property_names_and_strings_round_trip() -> Result<()> {
    let controls: String = ('\0'..='\u{1f}').collect();
    for text in [
        "ordinary",
        "\"quoted\"",
        "C:\\new\\table",
        "\\n\\u0041",
        "café 東京 🌍",
        &controls,
    ] {
        let bytes = feature(&[(text, ColumnValue::String(text))])?;
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["properties"][text], text);
        assert_eq!(value["properties"].as_object().unwrap().len(), 1);
        assert_eq!(
            value["geometry"],
            json!({"type": "Point", "coordinates": [1, 2]})
        );
    }
    Ok(())
}

#[test]
fn collection_name_round_trips() -> Result<()> {
    let name = "a \"collection\"\\name\n\t東京\u{0}";
    let mut bytes = Vec::new();
    let mut writer = GeoJsonWriter::new(&mut bytes);
    writer.dataset_begin(Some(name))?;
    writer.dataset_end()?;
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(
        value,
        json!({"type": "FeatureCollection", "name": name, "features": []})
    );
    Ok(())
}

#[test]
fn datetime_and_raw_json_keep_their_types() -> Result<()> {
    let raw = r#"{"nested":[1,true,null,"line\nslash\\quote\""]}"#;
    let date = "2026-11-09T12:34:56+01:00";
    let bytes = feature(&[
        ("date\n\\", ColumnValue::DateTime(date)),
        ("raw\t\\", ColumnValue::Json(raw)),
    ])?;
    let value: Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(value["properties"]["date\n\\"], date);
    assert_eq!(
        value["properties"]["raw\t\\"],
        serde_json::from_str::<Value>(raw).unwrap()
    );
    Ok(())
}

#[test]
fn numeric_and_boolean_property_names_are_escaped() -> Result<()> {
    let values = [
        ColumnValue::Byte(-1),
        ColumnValue::UByte(2),
        ColumnValue::Bool(true),
        ColumnValue::Short(-3),
        ColumnValue::UShort(4),
        ColumnValue::Int(-5),
        ColumnValue::UInt(6),
        ColumnValue::Long(-7),
        ColumnValue::ULong(u64::MAX),
        ColumnValue::Float(1.25),
        ColumnValue::Double(-2.5),
    ];
    let expected = [
        json!(-1),
        json!(2),
        json!(true),
        json!(-3),
        json!(4),
        json!(-5),
        json!(6),
        json!(-7),
        json!(u64::MAX),
        json!(1.25),
        json!(-2.5),
    ];
    for (number, expected) in values.into_iter().zip(expected) {
        let bytes = feature(&[("number\n\\\"", number)])?;
        let value: Value = serde_json::from_slice(&bytes).unwrap();
        assert_eq!(value["properties"]["number\n\\\""], expected);
    }
    Ok(())
}

struct FailAfter(usize);

impl Write for FailAfter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        if self.0 == 0 {
            return Err(io::Error::new(io::ErrorKind::BrokenPipe, "writer stopped"));
        }
        let count = bytes.len().min(self.0);
        self.0 -= count;
        Ok(count)
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[test]
fn string_encoding_preserves_writer_errors() {
    // Fail inside the encoded key and inside the encoded value, respectively.
    for budget in [2, 11] {
        let mut writer = GeoJsonWriter::new(FailAfter(budget));
        let error = writer
            .property(0, "key", &ColumnValue::String("value\n"))
            .unwrap_err();
        match error {
            GeozeroError::IoError(error) => {
                assert_eq!(error.kind(), io::ErrorKind::BrokenPipe);
                assert_eq!(error.to_string(), "writer stopped");
            }
            other => panic!("expected original I/O error, got {other:?}"),
        }
    }
}
