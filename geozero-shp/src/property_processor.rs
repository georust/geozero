use crate::reader::ShapeRecord;
use dbase::FieldValue;
use geozero::error::Result;
use geozero::{ColumnValue, FeatureProperties, PropertyProcessor};

impl FeatureProperties for ShapeRecord {
    /// Process feature properties.
    fn process_properties<P: PropertyProcessor>(&self, processor: &mut P) -> Result<bool> {
        let mut finish = false;
        let mut null_offset = 0usize;
        for (i, (name, value)) in self.record.as_ref().iter().enumerate() {
            let i = i - null_offset;
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
                FieldValue::Currency(val) => {
                    finish = processor.property(i, name, &ColumnValue::Double(*val))?;
                }
                FieldValue::DateTime(_) => {
                    let s = format!("{}", value);
                    finish = processor.property(i, name, &ColumnValue::DateTime(&s))?;
                }
                FieldValue::Memo(val) => {
                    finish = processor.property(i, name, &ColumnValue::String(val))?;
                }
                FieldValue::Character(None)
                | FieldValue::Numeric(None)
                | FieldValue::Logical(None)
                | FieldValue::Date(None)
                | FieldValue::Float(None) => { null_offset += 1 } // Ignore NULL values
            }
            if finish {
                break;
            }
        }
        Ok(finish)
    }
}
