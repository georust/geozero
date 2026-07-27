use std::io::Write as _;

use diesel::deserialize::{self, FromSql};
use diesel::pg::{self, Pg};
use diesel::serialize::{self, IsNull, Output, ToSql};

use super::sql_types;
use crate::GeozeroGeometry;
use crate::wkb::{self, Ewkb, FromWkb};

impl<B: AsRef<[u8]> + std::fmt::Debug> ToSql<sql_types::Geometry, Pg> for Ewkb<B> {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(self.0.as_ref())?;
        Ok(IsNull::No)
    }
}

impl<B: AsRef<[u8]> + std::fmt::Debug> ToSql<sql_types::Geography, Pg> for Ewkb<B> {
    fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
        out.write_all(self.0.as_ref())?;
        Ok(IsNull::No)
    }
}

impl FromSql<sql_types::Geometry, Pg> for Ewkb<Vec<u8>> {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(bytes.as_bytes().to_vec()))
    }
}

impl FromSql<sql_types::Geography, Pg> for Ewkb<Vec<u8>> {
    fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
        Ok(Self(bytes.as_bytes().to_vec()))
    }
}

/// `geometry` and `geography` are distinct SQL types but share the same EWKB wire format.
macro_rules! impl_wkb_conversions {
    ($($sql_type:ty),+) => {
        $(
            impl<T: FromWkb + Sized + std::fmt::Debug> FromSql<$sql_type, Pg> for wkb::Decode<T> {
                fn from_sql(bytes: pg::PgValue) -> deserialize::Result<Self> {
                    let mut rdr = std::io::Cursor::new(bytes.as_bytes());
                    let geom = T::from_wkb(&mut rdr, wkb::WkbDialect::Ewkb)?;
                    Ok(wkb::Decode {
                        geometry: Some(geom),
                    })
                }
            }

            impl<T: GeozeroGeometry + Sized + std::fmt::Debug> ToSql<$sql_type, Pg> for wkb::Encode<T> {
                fn to_sql(&self, out: &mut Output<Pg>) -> serialize::Result {
                    let mut wkb_out: Vec<u8> = Vec::new();
                    let mut writer = wkb::WkbWriter::with_opts(
                        &mut wkb_out,
                        wkb::WkbDialect::Ewkb,
                        self.0.dims(),
                        self.0.srid(),
                        Vec::new(),
                    );
                    self.0.process_geom(&mut writer)?;
                    out.write_all(&wkb_out)?;
                    Ok(IsNull::No)
                }
            }
        )+
    };
}

impl_wkb_conversions!(sql_types::Geometry, sql_types::Geography);

/// Implement Diesel `FromSql<Geometry>` for a type implementing `FromWkb`.
///
/// CAUTION: Does not support decoding NULL values!
#[macro_export]
macro_rules! impl_diesel_postgis_decode {
    ( $t:ty ) => {
        impl
            diesel::deserialize::FromSql<
                $crate::postgis::diesel::sql_types::Geometry,
                diesel::pg::Pg,
            > for $t
        {
            fn from_sql(bytes: diesel::pg::PgValue) -> diesel::deserialize::Result<Self> {
                use $crate::wkb::FromWkb;
                let mut rdr = std::io::Cursor::new(bytes.as_bytes());
                let geom = <$t>::from_wkb(&mut rdr, $crate::wkb::WkbDialect::Ewkb)?;
                Ok(geom)
            }
        }
    };
}

/// Implement Diesel `ToSql<Geometry>` for a type implementing `GeozeroGeometry`.
#[macro_export]
macro_rules! impl_diesel_postgis_encode {
    ( $t:ty ) => {
        impl diesel::serialize::ToSql<$crate::postgis::diesel::sql_types::Geometry, diesel::pg::Pg>
            for $t
        {
            fn to_sql(
                &self,
                out: &mut diesel::serialize::Output<diesel::pg::Pg>,
            ) -> diesel::serialize::Result {
                use std::io::Write;

                use $crate::GeozeroGeometry;
                let mut wkb_out: Vec<u8> = Vec::new();
                let mut writer = $crate::wkb::WkbWriter::with_opts(
                    &mut wkb_out,
                    $crate::wkb::WkbDialect::Ewkb,
                    self.dims(),
                    self.srid(),
                    Vec::new(),
                );
                self.process_geom(&mut writer)?;
                out.write_all(&wkb_out)?;
                Ok(diesel::serialize::IsNull::No)
            }
        }
    };
}
