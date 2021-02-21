/// PostGIS geometry type encoding/decoding for [rust-postgres](https://github.com/sfackler/rust-postgres).
#[cfg(feature = "postgis-postgres")]
pub mod postgres {
    use crate::wkb::{self, FromWkb};
    use crate::{GeozeroGeometry, ToWkt};
    use bytes::{BufMut, BytesMut};
    use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
    use std::fmt;

    impl<T: GeozeroGeometry + FromWkb + Sized> FromSql<'_> for wkb::Geometry<T> {
        fn from_sql(
            _ty: &Type,
            raw: &[u8],
        ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
            let mut rdr = std::io::Cursor::new(raw);
            let geom = T::from_wkb(&mut rdr, wkb::WkbDialect::Ewkb)?;
            Ok(wkb::Geometry(geom))
        }

        fn accepts(ty: &Type) -> bool {
            match ty.name() {
                "geography" | "geometry" => true,
                _ => false,
            }
        }
    }

    // required by ToSql
    impl<'a, T: GeozeroGeometry + Sized> fmt::Debug for wkb::Geometry<T> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str(&self.0.to_wkt().unwrap_or("<unkown geometry>".to_string()))
        }
    }

    impl<'a, T: GeozeroGeometry + Sized> ToSql for wkb::Geometry<T> {
        fn to_sql(
            &self,
            _ty: &Type,
            out: &mut BytesMut,
        ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
            let pgout = &mut out.writer();
            let mut writer = wkb::WkbWriter::new(pgout, wkb::WkbDialect::Ewkb);
            writer.dims = self.0.dims();
            writer.srid = self.0.srid();
            self.0.process_geom(&mut writer)?;
            Ok(IsNull::No)
        }

        fn accepts(ty: &Type) -> bool {
            match ty.name() {
                "geography" | "geometry" => true,
                _ => false,
            }
        }

        to_sql_checked!();
    }
}

/// PostGIS geometry type encoding/decoding for [SQLx](https://github.com/launchbadge/sqlx).
#[cfg(feature = "postgis-sqlx")]
pub mod sqlx {
    use crate::wkb::{self, FromWkb};
    use crate::GeozeroGeometry;
    use sqlx::decode::Decode;
    use sqlx::encode::{Encode, IsNull};
    use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres};
    use sqlx::ValueRef;

    type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

    impl<T: GeozeroGeometry + Sized> sqlx::Type<Postgres> for wkb::Geometry<T> {
        fn type_info() -> PgTypeInfo {
            PgTypeInfo::with_name("geometry")
        }
    }

    impl<'de, T: GeozeroGeometry + FromWkb + Sized> Decode<'de, Postgres> for wkb::Geometry<T> {
        fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
            if value.is_null() {
                // Return empty geometry
                return Ok(wkb::Geometry(T::empty()));
            }
            let mut blob = <&[u8] as Decode<Postgres>>::decode(value)?;
            let geom = T::from_wkb(&mut blob, wkb::WkbDialect::Ewkb)
                .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
            Ok(wkb::Geometry(geom))
        }
    }

    impl<T: GeozeroGeometry + Sized> Encode<'_, Postgres> for wkb::Geometry<T> {
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
}
