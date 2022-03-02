use crate::reader::ShapeRecord;
use dbase::FieldValue;
use geozero::error::Result;
use geozero::{ColumnValue, FeatureProperties, PropertyProcessor};

impl FeatureProperties for ShapeRecord {
    /// Process feature properties.
    fn process_properties<P: PropertyProcessor>(&self, processor: &mut P) -> Result<bool> {
        let mut finish = false;
        for (i, (name, value)) in self.record.clone().into_iter().enumerate() {
            match value {
                FieldValue::Character(Some(val)) => {
                    finish = processor.property(i, &name, &ColumnValue::String(&val))?;
                }
                FieldValue::Numeric(Some(val)) => {
                    finish = processor.property(i, &name, &ColumnValue::Double(val))?;
                }
                FieldValue::Logical(Some(val)) => {
                    finish = processor.property(i, &name, &ColumnValue::Bool(val))?;
                }
                FieldValue::Date(Some(_)) => {
                    let s = format!("{}", value);
                    finish = processor.property(i, &name, &ColumnValue::DateTime(&s))?;
                }
                FieldValue::Float(Some(val)) => {
                    finish = processor.property(i, &name, &ColumnValue::Float(val))?;
                }
                FieldValue::Integer(val) => {
                    finish = processor.property(i, &name, &ColumnValue::Int(val))?;
                }
                FieldValue::Double(val) => {
                    finish = processor.property(i, &name, &ColumnValue::Double(val))?;
                }
                _ => {}
            }
            if finish {
                break;
            }
        }
        Ok(finish)
    }
}
