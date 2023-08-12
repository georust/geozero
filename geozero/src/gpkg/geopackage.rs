use crate::wkb::{self, FromWkb};
use crate::GeozeroGeometry;
use sqlx::decode::Decode;
use sqlx::encode::{Encode, IsNull};
use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::ValueRef;
use std::borrow::Cow;

type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

impl<T: FromWkb + Sized> sqlx::Type<Sqlite> for wkb::Decode<T> {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'de, T: FromWkb + Sized> Decode<'de, Sqlite> for wkb::Decode<T> {
    fn decode(value: SqliteValueRef<'de>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(wkb::Decode { geometry: None });
        }
        let mut blob = <&[u8] as Decode<Sqlite>>::decode(value)?;
        let geom = T::from_wkb(&mut blob, wkb::WkbDialect::Geopackage)
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        Ok(wkb::Decode {
            geometry: Some(geom),
        })
    }
}

impl sqlx::Type<Sqlite> for wkb::GpkgWkb {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'de> Decode<'de, Sqlite> for wkb::GpkgWkb {
    fn decode(value: SqliteValueRef<'de>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(wkb::GpkgWkb(Vec::new()));
        }
        let blob = <&[u8] as Decode<Sqlite>>::decode(value)?;
        Ok(wkb::GpkgWkb(blob.to_vec()))
    }
}

impl<T: GeozeroGeometry + Sized> sqlx::Type<Sqlite> for wkb::Encode<T> {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q, T: GeozeroGeometry + Sized> Encode<'q, Sqlite> for wkb::Encode<T> {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = wkb::WkbWriter::with_opts(
            &mut wkb_out,
            wkb::WkbDialect::Geopackage,
            self.0.dims(),
            self.0.srid(),
            Vec::new(),
        );
        self.0
            .process_geom(&mut writer)
            .expect("Failed to encode Geometry");
        args.push(SqliteArgumentValue::Blob(Cow::Owned(wkb_out)));
        IsNull::No
    }
}

// Same as macros for geometry types without wrapper
// Limitations:
// - Can only be used with self defined types
// - Decode does not support NULL values

/// impl `sqlx::Type` for geometry type
#[macro_export]
macro_rules! impl_sqlx_gpkg_type_info {
    ( $t:ty ) => {
        impl sqlx::Type<sqlx::sqlite::Sqlite> for $t {
            fn type_info() -> sqlx::sqlite::SqliteTypeInfo {
                <Vec<u8> as sqlx::Type<sqlx::sqlite::Sqlite>>::type_info()
            }
        }
    };
}

/// impl `sqlx::decode::Decode` for geometry type implementing `FromWkb`
///
/// CAUTION: Does not support decoding NULL value!
#[macro_export]
macro_rules! impl_sqlx_gpkg_decode {
    ( $t:ty ) => {
        impl<'de> sqlx::decode::Decode<'de, sqlx::sqlite::Sqlite> for $t {
            fn decode(
                value: sqlx::sqlite::SqliteValueRef<'de>,
            ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                use sqlx::ValueRef;
                use $crate::wkb::FromWkb;
                if value.is_null() {
                    return Err(Box::new(sqlx::Error::Decode(
                        "Cannot decode NULL value".into(),
                    )));
                }
                let mut blob =
                    <&[u8] as sqlx::decode::Decode<sqlx::sqlite::Sqlite>>::decode(value)?;
                let geom = <$t>::from_wkb(&mut blob, $crate::wkb::WkbDialect::Ewkb)
                    .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                Ok(geom)
            }
        }
    };
}

/// impl `sqlx::decode::Decode` for geometry type implementing `GeozeroGeometry`
#[macro_export]
macro_rules! impl_sqlx_gpkg_encode {
    ( $t:ty ) => {
        impl<'q> sqlx::encode::Encode<'q, sqlx::sqlite::Sqlite> for $t {
            fn encode_by_ref(
                &self,
                args: &mut Vec<sqlx::sqlite::SqliteArgumentValue<'q>>,
            ) -> sqlx::encode::IsNull {
                use $crate::GeozeroGeometry;
                let mut wkb_out: Vec<u8> = Vec::new();
                let mut writer = $crate::wkb::WkbWriter::with_opts(
                    &mut wkb_out,
                    $crate::wkb::WkbDialect::Geopackage,
                    self.dims(),
                    self.srid(),
                    Vec::new(),
                );
                self.process_geom(&mut writer)
                    .expect("Failed to encode Geometry");
                args.push(sqlx::sqlite::SqliteArgumentValue::Blob(
                    std::borrow::Cow::Owned(wkb_out),
                ));
                sqlx::encode::IsNull::No
            }
        }
    };
}
