/// Geopackage geometry type encoding/decoding.
use crate::wkb::{self, FromWkb};
use crate::GeozeroGeometry;
use sqlx::decode::Decode;
use sqlx::encode::{Encode, IsNull};
use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
use sqlx::ValueRef;
use std::borrow::Cow;

type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

impl<T: GeozeroGeometry + Sized> sqlx::Type<Sqlite> for wkb::Geometry<T> {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
    }
}

impl<'de, T: GeozeroGeometry + FromWkb + Sized> Decode<'de, Sqlite> for wkb::Geometry<T> {
    fn decode(value: SqliteValueRef<'de>) -> Result<Self, BoxDynError> {
        if value.is_null() {
            // Return empty geometry
            return Ok(wkb::Geometry(T::empty()));
        }
        let mut blob = <&[u8] as Decode<Sqlite>>::decode(value)?;
        let geom = T::from_wkb(&mut blob, wkb::WkbDialect::Geopackage)
            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
        Ok(wkb::Geometry(geom))
    }
}

impl<'q, T: GeozeroGeometry + Sized> Encode<'q, Sqlite> for wkb::Geometry<T> {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = wkb::WkbWriter::new(&mut wkb_out, wkb::WkbDialect::Geopackage);
        writer.dims = self.0.dims();
        writer.srid = self.0.srid();
        GeozeroGeometry::process_geom(&self.0, &mut writer).expect("Failed to encode Geometry");
        args.push(SqliteArgumentValue::Blob(Cow::Owned(wkb_out)));
        IsNull::No
    }
}
