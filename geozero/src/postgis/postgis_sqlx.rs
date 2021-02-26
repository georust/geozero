use crate::wkb::{self, FromWkb};
use crate::GeozeroGeometry;
use sqlx::decode::Decode;
use sqlx::encode::{Encode, IsNull};
use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres};
use sqlx::ValueRef;

type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

impl<T: FromWkb + Sized> sqlx::Type<Postgres> for wkb::Decode<T> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
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

impl<T: GeozeroGeometry + Sized> sqlx::Type<Postgres> for wkb::Encode<T> {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("geometry")
    }
}

impl<T: GeozeroGeometry + Sized> Encode<'_, Postgres> for wkb::Encode<T> {
    fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
        let mut wkb_out: Vec<u8> = Vec::new();
        let mut writer = wkb::WkbWriter::new(&mut wkb_out, wkb::WkbDialect::Ewkb);
        writer.dims = self.0.dims();
        writer.srid = self.0.srid();
        self.0
            .process_geom(&mut writer)
            .expect("Failed to encode Geometry");
        buf.extend(&wkb_out); // Is there a way to write directly into PgArgumentBuffer?

        IsNull::No
    }
}
