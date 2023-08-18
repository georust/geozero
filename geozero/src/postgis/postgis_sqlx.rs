use crate::wkb::{self, FromWkb};
use crate::GeozeroGeometry;
use sqlx::decode::Decode;
use sqlx::encode::{Encode, IsNull};
use sqlx::postgres::{PgArgumentBuffer, PgHasArrayType, PgTypeInfo, PgValueRef, Postgres};
use sqlx::ValueRef;

type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

impl<T: FromWkb + Sized> sqlx::Type<Postgres> for wkb::Decode<T> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

impl<T: FromWkb + Sized> PgHasArrayType for wkb::Decode<T> {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_geometry")
    }
}

impl<'de, T: FromWkb + Sized> Decode<'de, Postgres> for wkb::Decode<T> {
    fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(wkb::Decode { geometry: None });
        }
        let mut blob = <&[u8] as Decode<Postgres>>::decode(value)?;
        let geom = T::from_wkb(&mut blob, wkb::WkbDialect::Ewkb)
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        Ok(wkb::Decode {
            geometry: Some(geom),
        })
    }
}

impl sqlx::Type<Postgres> for wkb::Ewkb {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

impl PgHasArrayType for wkb::Ewkb {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_geometry")
    }
}

impl<'de> Decode<'de, Postgres> for wkb::Ewkb {
    fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            return Ok(wkb::Ewkb(Vec::new()));
        }
        let blob = <&[u8] as Decode<Postgres>>::decode(value)?;
        Ok(wkb::Ewkb(blob.to_vec()))
    }
}

impl<T: GeozeroGeometry + Sized> sqlx::Type<Postgres> for wkb::Encode<T> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

impl<T: GeozeroGeometry + Sized> PgHasArrayType for wkb::Encode<T> {
    fn array_type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("_geometry")
    }
}

impl<T: GeozeroGeometry + Sized> Encode<'_, Postgres> for wkb::Encode<T> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = wkb::WkbWriter::with_opts(
            &mut wkb_out,
            wkb::WkbDialect::Ewkb,
            self.0.dims(),
            self.0.srid(),
            Vec::new(),
        );
        self.0
            .process_geom(&mut writer)
            .expect("Failed to encode Geometry");
        buf.extend(&wkb_out); // Is there a way to write directly into PgArgumentBuffer?

        IsNull::No
    }
}

// Same as macros for geometry types without wrapper
// Limitations:
// - Can only be used with self defined types
// - Decode does not support NULL values

/// impl `sqlx::Type` and  `PgHasArrayType` for geometry type
#[macro_export]
macro_rules! impl_sqlx_postgis_type_info {
    ( $t:ty ) => {
        impl sqlx::Type<sqlx::postgres::Postgres> for $t {
            fn type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("geometry")
            }
        }

        impl sqlx::postgres::PgHasArrayType for $t {
            fn array_type_info() -> sqlx::postgres::PgTypeInfo {
                sqlx::postgres::PgTypeInfo::with_name("_geometry")
            }
        }
    };
}

/// impl `sqlx::decode::Decode` for geometry type implementing `FromWkb`
///
/// CAUTION: Does not support decoding NULL value!
#[macro_export]
macro_rules! impl_sqlx_postgis_decode {
    ( $t:ty ) => {
        impl<'de> sqlx::decode::Decode<'de, sqlx::postgres::Postgres> for $t {
            fn decode(
                value: sqlx::postgres::PgValueRef<'de>,
            ) -> std::result::Result<Self, Box<dyn std::error::Error + Send + Sync>> {
                use sqlx::ValueRef;
                use $crate::wkb::FromWkb;
                if value.is_null() {
                    return Err(Box::new(sqlx::Error::Decode(
                        "Cannot decode NULL value".into(),
                    )));
                }
                let mut blob =
                    <&[u8] as sqlx::decode::Decode<sqlx::postgres::Postgres>>::decode(value)?;
                let geom = <$t>::from_wkb(&mut blob, $crate::wkb::WkbDialect::Ewkb)
                    .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                Ok(geom)
            }
        }
    };
}

/// impl `sqlx::decode::Decode` for geometry type implementing `GeozeroGeometry`
#[macro_export]
macro_rules! impl_sqlx_postgis_encode {
    ( $t:ty ) => {
        impl sqlx::encode::Encode<'_, sqlx::postgres::Postgres> for $t {
            fn encode_by_ref(
                &self,
                buf: &mut sqlx::postgres::PgArgumentBuffer,
            ) -> sqlx::encode::IsNull {
                use $crate::GeozeroGeometry;
                let mut wkb_out: Vec<u8> = Vec::new();
                let mut writer = $crate::wkb::WkbWriter::with_opts(
                    &mut wkb_out,
                    $crate::wkb::WkbDialect::Ewkb,
                    self.dims(),
                    self.srid(),
                    Vec::new(),
                );
                self.process_geom(&mut writer)
                    .expect("Failed to encode Geometry");
                buf.extend(&wkb_out); // Is there a way to write directly into PgArgumentBuffer?

                sqlx::encode::IsNull::No
            }
        }
    };
}
