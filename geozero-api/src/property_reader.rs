use std::collections::HashMap;

#[derive(PartialEq, Debug)]
pub enum ColumnValue<'a> {
    Byte(i8),
    UByte(u8),
    Bool(bool),
    Short(i16),
    UShort(u16),
    Int(i32),
    UInt(u32),
    Long(i64),
    ULong(u64),
    Float(f32),
    Double(f64),
    String(&'a str),
    Json(&'a str),
    DateTime(&'a str),
    Binary(&'a [u8]),
}

pub trait PropertyReader {
    fn property(&mut self, _idx: usize, _name: &str, _value: ColumnValue) -> bool {
        true
    }
}

impl PropertyReader for HashMap<String, String> {
    fn property(&mut self, _idx: usize, colname: &str, colval: ColumnValue) -> bool {
        let vstr = match colval {
            ColumnValue::Byte(v) => v.to_string(),
            ColumnValue::UByte(v) => v.to_string(),
            ColumnValue::Bool(v) => v.to_string(),
            ColumnValue::Short(v) => v.to_string(),
            ColumnValue::UShort(v) => v.to_string(),
            ColumnValue::Int(v) => v.to_string(),
            ColumnValue::UInt(v) => v.to_string(),
            ColumnValue::Long(v) => v.to_string(),
            ColumnValue::ULong(v) => v.to_string(),
            ColumnValue::Float(v) => v.to_string(),
            ColumnValue::Double(v) => v.to_string(),
            ColumnValue::String(v) => v.to_string(),
            ColumnValue::Json(v) => v.to_string(),
            ColumnValue::DateTime(v) => v.to_string(),
            ColumnValue::Binary(_v) => "[BINARY]".to_string(),
        };
        self.insert(colname.to_string(), vstr);
        false
    }
}
