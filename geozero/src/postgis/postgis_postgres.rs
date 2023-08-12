use crate::wkb::{self, FromWkb};
use crate::GeozeroGeometry;
use bytes::{BufMut, BytesMut};
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};

impl<T: FromWkb + Sized> FromSql<'_> for wkb::Decode<T> {
    fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut rdr = std::io::Cursor::new(raw);
        let geom = T::from_wkb(&mut rdr, wkb::WkbDialect::Ewkb)?;
        Ok(wkb::Decode {
            geometry: Some(geom),
        })
    }

    fn from_sql_null(_ty: &Type) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(wkb::Decode { geometry: None })
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty.name(), "geography" | "geometry")
    }
}

impl FromSql<'_> for wkb::Ewkb {
    fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(wkb::Ewkb(raw.to_vec()))
    }

    fn from_sql_null(_ty: &Type) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        Ok(wkb::Ewkb(Vec::new()))
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty.name(), "geography" | "geometry")
    }
}

impl<T: GeozeroGeometry + Sized> ToSql for wkb::Encode<T> {
    fn to_sql(
        &self,
        _ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        let pgout = &mut out.writer();
        let mut writer = wkb::WkbWriter::with_opts(
            pgout,
            wkb::WkbDialect::Ewkb,
            self.0.dims(),
            self.0.srid(),
            Vec::new(),
        );
        self.0.process_geom(&mut writer)?;
        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        matches!(ty.name(), "geography" | "geometry")
    }

    to_sql_checked!();
}

// Same as macros for geometry types without wrapper
// Limitations:
// - Can only be used with self defined types
// - Decode does not support NULL values

/// impl `FromSql` for geometry type implementing `FromWkb`
///
/// CAUTION: Does not support decoding NULL value!
#[macro_export]
macro_rules! impl_postgres_postgis_decode {
    ( $t:ty ) => {
        impl postgres_types::FromSql<'_> for $t {
            fn from_sql(
                _ty: &postgres_types::Type,
                raw: &[u8],
            ) -> std::result::Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                use $crate::wkb::FromWkb;
                let mut rdr = std::io::Cursor::new(raw);
                let geom = <$t>::from_wkb(&mut rdr, $crate::wkb::WkbDialect::Ewkb)?;
                Ok(geom)
            }

            fn accepts(ty: &postgres_types::Type) -> bool {
                match ty.name() {
                    "geography" | "geometry" => true,
                    _ => false,
                }
            }
        }
    };
}

/// impl `ToSql` for geometry type implementing `GeozeroGeometry`
#[macro_export]
macro_rules! impl_postgres_postgis_encode {
    ( $t:ty ) => {
        impl<'a> postgres_types::ToSql for $t {
            fn to_sql(
                &self,
                _ty: &postgres_types::Type,
                out: &mut bytes::BytesMut,
            ) -> std::result::Result<postgres_types::IsNull, Box<dyn std::error::Error + Sync + Send>>
            {
                use $crate::GeozeroGeometry;
                use bytes::BufMut;

                let pgout = &mut out.writer();
                let mut writer = $crate::wkb::WkbWriter::with_opts(
                    pgout,
                    $crate::wkb::WkbDialect::Ewkb,
                    self.dims(),
                    self.srid(),
                    Vec::new(),
                );
                self.process_geom(&mut writer)?;
                Ok(postgres_types::IsNull::No)
            }

            fn accepts(ty: &postgres_types::Type) -> bool {
                match ty.name() {
                    "geography" | "geometry" => true,
                    _ => false,
                }
            }

            postgres_types::to_sql_checked!();
        }
    };
}
