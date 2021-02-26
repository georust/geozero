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

impl<T: GeozeroGeometry + Sized> sqlx::Type<Sqlite> for wkb::Encode<T> {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'q, T: GeozeroGeometry + Sized> Encode<'q, Sqlite> for wkb::Encode<T> {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = wkb::WkbWriter::new(&mut wkb_out, wkb::WkbDialect::Geopackage);
        writer.dims = self.0.dims();
        writer.srid = self.0.srid();
        self.0
            .process_geom(&mut writer)
            .expect("Failed to encode Geometry");
        args.push(SqliteArgumentValue::Blob(Cow::Owned(wkb_out)));
        IsNull::No
    }
}
