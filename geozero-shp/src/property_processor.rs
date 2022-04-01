use crate::reader::ShapeRecord;
use dbase::FieldValue;
use geozero::error::Result;
use geozero::{ColumnValue, FeatureProperties, PropertyProcessor};

impl FeatureProperties for ShapeRecord {
    /// Process feature properties.
    fn process_properties<P: PropertyProcessor>(&self, processor: &mut P) -> Result<bool> {
        let mut i = 0;
        for (name, value) in self.record.as_ref().iter() {
            let finish = match value {
                FieldValue::Character(Some(val)) => {
                    processor.property(i, name, &ColumnValue::String(val))?
                }
                FieldValue::Numeric(Some(val)) => {
                    processor.property(i, name, &ColumnValue::Double(*val))?
                }
                FieldValue::Logical(Some(val)) => {
                    processor.property(i, name, &ColumnValue::Bool(*val))?
                }
                FieldValue::Date(Some(_)) => {
                    let s = format!("{}", value);
                    processor.property(i, name, &ColumnValue::DateTime(&s))?
                }
                FieldValue::Float(Some(val)) => {
                    processor.property(i, name, &ColumnValue::Float(*val))?
                }
                FieldValue::Integer(val) => processor.property(i, name, &ColumnValue::Int(*val))?,
                FieldValue::Double(val) => {
                    processor.property(i, name, &ColumnValue::Double(*val))?
                }
                FieldValue::Currency(val) => {
                    processor.property(i, name, &ColumnValue::Double(*val))?
                }
                FieldValue::DateTime(_) => {
                    let s = format!("{}", value);
                    processor.property(i, name, &ColumnValue::DateTime(&s))?
                }
                FieldValue::Memo(val) => processor.property(i, name, &ColumnValue::String(val))?,
                FieldValue::Character(None)
                | FieldValue::Numeric(None)
                | FieldValue::Logical(None)
                | FieldValue::Date(None)
                | FieldValue::Float(None) => {
                    continue; // Ignore NULL values
                }
            };
            if finish {
                return Ok(true);
            }
            i += 1;
        }
        Ok(false)
    }
}
