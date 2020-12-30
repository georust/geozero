use dbase::FieldValue;
use geozero::error::Result;
use geozero::{ColumnValue, PropertyProcessor};
use std::collections::HashMap;

pub fn process_properties<P: PropertyProcessor>(
    record: &dbase::Record,
    processor: &mut P,
) -> Result<bool> {
    let mut finish = false;
    for (i, (name, value)) in record.iter().enumerate() {
        match value {
            FieldValue::Character(Some(val)) => {
                finish = processor.property(i, name, &ColumnValue::String(val))?;
            }
            FieldValue::Numeric(Some(val)) => {
                finish = processor.property(i, name, &ColumnValue::Double(*val))?;
            }
            FieldValue::Logical(Some(val)) => {
                finish = processor.property(i, name, &ColumnValue::Bool(*val))?;
            }
            FieldValue::Date(Some(_)) => {
                let s = format!("{}", value);
                finish = processor.property(i, name, &ColumnValue::DateTime(&s))?;
            }
            FieldValue::Float(Some(val)) => {
                finish = processor.property(i, name, &ColumnValue::Float(*val))?;
            }
            FieldValue::Integer(val) => {
                finish = processor.property(i, name, &ColumnValue::Int(*val))?;
            }
            FieldValue::Double(val) => {
                finish = processor.property(i, name, &ColumnValue::Double(*val))?;
            }
            _ => {}
        }
        if finish {
            break;
        }
    }
    Ok(finish)
}

/// Return all properties in a HashMap
pub fn properties(record: &dbase::Record) -> Result<HashMap<String, String>> {
    let mut properties = HashMap::new();
    let _ = process_properties(record, &mut properties)?;
    Ok(properties)
}
