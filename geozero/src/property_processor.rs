use crate::error::{GeozeroError, Result};
use std::collections::HashMap;
use std::fmt;

/// Feature property value.
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

/// Feature property processing trait.
///
/// # Usage example:
///
/// ```rust
/// use geozero::{PropertyProcessor, ColumnValue, error::Result};
///
/// struct PropertyPrinter;
///
/// impl PropertyProcessor for PropertyPrinter {
///     fn property(&mut self, i: usize, n: &str, v: &ColumnValue) -> Result<bool> {
///         println!("columnidx: {} name: {} value: {:?}", i, n, v);
///         Ok(false) // don't abort
///     }
/// }
/// ```
#[allow(unused_variables)]
pub trait PropertyProcessor {
    /// Process property value. Abort processing, if return value is true.
    fn property(&mut self, idx: usize, name: &str, value: &ColumnValue) -> Result<bool> {
        Ok(true)
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

#[doc(hidden)]
pub struct PropertyReader<'a, T: PropertyReadType> {
    pub name: &'a str,
    pub value: Result<T>,
}

#[doc(hidden)]
pub struct PropertyReaderIdx<T: PropertyReadType> {
    pub idx: usize,
    pub value: Result<T>,
}

/// Get property value as Rust type.
pub trait PropertyReadType<T = Self>
where
    T: PropertyReadType,
{
    /// Get property value as Rust type.
    fn get_value(v: &ColumnValue) -> Result<T>;
}

impl<T: PropertyReadType> PropertyProcessor for PropertyReader<'_, T> {
    fn property(&mut self, _i: usize, name: &str, v: &ColumnValue) -> Result<bool> {
        if name == self.name {
            self.value = T::get_value(v);
            Ok(true) // finish
        } else {
            Ok(false)
        }
    }
}

impl<T: PropertyReadType> PropertyProcessor for PropertyReaderIdx<T> {
    fn property(&mut self, i: usize, _name: &str, v: &ColumnValue) -> Result<bool> {
        if i == self.idx {
            self.value = T::get_value(v);
            Ok(true) // finish
        } else {
            Ok(false)
        }
    }
}

macro_rules! impl_scalar_property_reader {
    ( $t:ty, $e:path ) => {
        impl From<&ColumnValue<'_>> for Result<$t> {
            fn from(v: &ColumnValue) -> Result<$t> {
                if let $e(v) = v {
                    Ok(*v)
                } else {
                    Err(GeozeroError::ColumnType(stringify!($e).to_string(), format!("{:?}", v)))
                }
            }
        }
        impl PropertyReadType for $t {
            fn get_value(v: &ColumnValue) -> Result<$t> {
                v.into()
            }
        }
    };
}

impl_scalar_property_reader!(i8, ColumnValue::Byte);
impl_scalar_property_reader!(u8, ColumnValue::UByte);
impl_scalar_property_reader!(bool, ColumnValue::Bool);
impl_scalar_property_reader!(i16, ColumnValue::Short);
impl_scalar_property_reader!(u16, ColumnValue::UShort);
impl_scalar_property_reader!(i32, ColumnValue::Int);
impl_scalar_property_reader!(u32, ColumnValue::UInt);
impl_scalar_property_reader!(i64, ColumnValue::Long);
impl_scalar_property_reader!(u64, ColumnValue::ULong);
impl_scalar_property_reader!(f32, ColumnValue::Float);
impl_scalar_property_reader!(f64, ColumnValue::Double);

impl From<&ColumnValue<'_>> for Result<String> {
    fn from(v: &ColumnValue) -> Result<String> {
        Ok(v.to_string())
    }
}

impl PropertyReadType for String {
    fn get_value(v: &ColumnValue) -> Result<String> {
        v.into()
    }
}

impl PropertyProcessor for HashMap<String, String> {
    fn property(&mut self, _idx: usize, colname: &str, colval: &ColumnValue) -> Result<bool> {
        self.insert(colname.to_string(), colval.to_string());
        Ok(false)
    }
}

#[test]
fn convert_column_value() {
    let v = &ColumnValue::Int(42);
    assert_eq!(Result::<i32>::from(v).unwrap(), 42);
    assert_eq!(
        Result::<i64>::from(v).unwrap_err().to_string(),
        "expected a `ColumnValue::Long` value but found `Int(42)`"
    );
    assert_eq!(Result::<String>::from(v).unwrap(), "42".to_string());

    let v = &ColumnValue::String("Yes");
    assert_eq!(Result::<String>::from(v).unwrap(), "Yes".to_string());
    assert_eq!(
        Result::<i32>::from(v).unwrap_err().to_string(),
        "expected a `ColumnValue::Int` value but found `String(\"Yes\")`"
    );
}
