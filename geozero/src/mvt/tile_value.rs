use crate::mvt::tile::Value;
use crate::ColumnValue;
use std::hash::Hash;

/// A wrapper for the MVT value types.
#[derive(Debug, Clone, PartialEq)]
pub enum TileValue {
    Str(String),
    Float(f32),
    Double(f64),
    Int(i64),
    Uint(u64),
    Sint(i64),
    Bool(bool),
}

impl From<TileValue> for Value {
    fn from(tv: TileValue) -> Self {
        match tv {
            TileValue::Str(s) => Self {
                string_value: Some(s),
                ..Default::default()
            },
            TileValue::Float(f) => Self {
                float_value: Some(f),
                ..Default::default()
            },
            TileValue::Double(d) => Self {
                double_value: Some(d),
                ..Default::default()
            },
            TileValue::Int(i) => Self {
                int_value: Some(i),
                ..Default::default()
            },
            TileValue::Uint(u) => Self {
                uint_value: Some(u),
                ..Default::default()
            },
            TileValue::Sint(i) => Self {
                sint_value: Some(i),
                ..Default::default()
            },
            TileValue::Bool(b) => Self {
                bool_value: Some(b),
                ..Default::default()
            },
        }
    }
}

impl TryFrom<Value> for TileValue {
    type Error = ();

    fn try_from(v: Value) -> Result<Self, Self::Error> {
        Ok(if let Some(s) = v.string_value {
            Self::Str(s)
        } else if let Some(f) = v.float_value {
            Self::Float(f)
        } else if let Some(d) = v.double_value {
            Self::Double(d)
        } else if let Some(i) = v.int_value {
            Self::Int(i)
        } else if let Some(u) = v.uint_value {
            Self::Uint(u)
        } else if let Some(i) = v.sint_value {
            Self::Sint(i)
        } else if let Some(b) = v.bool_value {
            Self::Bool(b)
        } else {
            Err(())?
        })
    }
}

impl TryFrom<&ColumnValue<'_>> for TileValue {
    type Error = ();

    fn try_from(v: &ColumnValue) -> Result<Self, Self::Error> {
        // string_value - ColumnValue::String
        // float_value - ColumnValue::Float
        // double_value - ColumnValue::Double
        // int_value - ColumnValue::Long
        // uint_value - ColumnValue::ULong
        // sint_value - ColumnValue::Long
        // bool_value - ColumnValue::Bool

        Ok(match v {
            ColumnValue::Byte(v) => TileValue::Sint(*v as i64),
            ColumnValue::UByte(v) => TileValue::Uint(*v as u64),
            ColumnValue::Bool(v) => TileValue::Bool(*v),
            ColumnValue::Short(v) => TileValue::Sint(*v as i64),
            ColumnValue::UShort(v) => TileValue::Uint(*v as u64),
            ColumnValue::Int(v) => TileValue::Sint(*v as i64),
            ColumnValue::UInt(v) => TileValue::Uint(*v as u64),
            ColumnValue::Long(v) => TileValue::Sint(*v),
            ColumnValue::ULong(v) => TileValue::Uint(*v),
            ColumnValue::Float(v) => TileValue::Float(*v),
            ColumnValue::Double(v) => TileValue::Double(*v),
            ColumnValue::String(v) => TileValue::Str(v.to_string()),
            ColumnValue::Json(v) => TileValue::Str(v.to_string()),
            ColumnValue::DateTime(v) => TileValue::Str(v.to_string()),
            ColumnValue::Binary(_) => Err(())?,
        })
    }
}

// Treat floats as bits so that we can use as keys.
// It is up to the users to ensure that the bits are not NaNs, or are consistent.

impl Eq for TileValue {}

impl Hash for TileValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Str(s) => s.hash(state),
            Self::Float(f) => f.to_bits().hash(state),
            Self::Double(d) => d.to_bits().hash(state),
            Self::Int(i) => i.hash(state),
            Self::Uint(u) => u.hash(state),
            Self::Sint(i) => i.hash(state),
            Self::Bool(b) => b.hash(state),
        }
    }
}
