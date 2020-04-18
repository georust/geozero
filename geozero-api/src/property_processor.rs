use crate::DebugReader;
use std::collections::HashMap;
use std::fmt;

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

pub trait PropertyProcessor {
    fn property(&mut self, _idx: usize, _name: &str, _value: &ColumnValue) -> bool {
        true
    }
}

impl fmt::Display for ColumnValue<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColumnValue::Byte(v) => write!(f, "{}", v),
            ColumnValue::UByte(v) => write!(f, "{}", v),
            ColumnValue::Bool(v) => write!(f, "{}", v),
            ColumnValue::Short(v) => write!(f, "{}", v),
            ColumnValue::UShort(v) => write!(f, "{}", v),
            ColumnValue::Int(v) => write!(f, "{}", v),
            ColumnValue::UInt(v) => write!(f, "{}", v),
            ColumnValue::Long(v) => write!(f, "{}", v),
            ColumnValue::ULong(v) => write!(f, "{}", v),
            ColumnValue::Float(v) => write!(f, "{}", v),
            ColumnValue::Double(v) => write!(f, "{}", v),
            ColumnValue::String(v) => write!(f, "{}", v),
            ColumnValue::Json(v) => write!(f, "{}", v),
            ColumnValue::DateTime(v) => write!(f, "{}", v),
            ColumnValue::Binary(_v) => write!(f, "[BINARY]"),
        }
    }
}

impl PropertyProcessor for DebugReader {} //TODO

impl PropertyProcessor for HashMap<String, String> {
    fn property(&mut self, _idx: usize, colname: &str, colval: &ColumnValue) -> bool {
        self.insert(colname.to_string(), colval.to_string());
        false
    }
}
