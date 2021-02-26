use crate::wkb::{self, FromWkb};
use crate::{GeozeroGeometry, ToWkt};
use bytes::{BufMut, BytesMut};
use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use std::fmt;

impl<T: FromWkb + Sized> FromSql<'_> for wkb::Decode<T> {
    fn from_sql(_ty: &Type, raw: &[u8]) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
        let mut rdr = std::io::Cursor::new(raw);
        let geom = T::from_wkb(&mut rdr, wkb::WkbDialect::Ewkb)?;
        Ok(wkb::Decode {
            geometry: Some(geom),
        })
    }

    fn accepts(ty: &Type) -> bool {
        match ty.name() {
            "geography" | "geometry" => true,
            _ => false,
        }
    }
}

// required by ToSql
impl<'a, T: GeozeroGeometry + Sized> fmt::Debug for wkb::Encode<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0.to_wkt().unwrap_or("<unkown geometry>".to_string()))
    }
}

impl<'a, T: GeozeroGeometry + Sized> ToSql for wkb::Encode<T> {
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
