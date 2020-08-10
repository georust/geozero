// This should be included in georust/geo to avoid a newtype
/// Geopackage geometry type decoding for [geo-types](https://github.com/georust/geo).
pub mod geo {
    use crate::geo_types::Geo;
    use crate::wkb;
    use sqlx::decode::Decode;
    use sqlx::sqlite::{Sqlite, SqliteTypeInfo, SqliteValueRef};

    type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

    /// geo-types wrapper for Geopackage geometry decoding.
    pub struct Geometry(pub geo_types::Geometry<f64>);

    impl sqlx::Type<Sqlite> for Geometry {
        fn type_info() -> SqliteTypeInfo {
            <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
        }
    }

    impl<'de> Decode<'de, Sqlite> for Geometry {
        fn decode(value: SqliteValueRef<'de>) -> Result<Self, BoxDynError> {
            let mut blob = <&[u8] as Decode<Sqlite>>::decode(value)?;
            let mut geo = Geo::new();
            wkb::process_gpkg_geom(&mut blob, &mut geo)
                .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
            let geom = Geometry(geo.geometry().to_owned());
            Ok(geom)
        }
    }
}

// This should be included in georust/geos to avoid a newtype
/// Geopackage geometry type encoding/decoding for [GEOS](https://github.com/georust/geos).
#[cfg(feature = "geos-lib")]
pub mod geos {
    use crate::geos::{process_geos, Geos};
    use crate::wkb;
    use geos::Geom;
    use sqlx::decode::Decode;
    use sqlx::encode::{Encode, IsNull};
    use sqlx::sqlite::{Sqlite, SqliteArgumentValue, SqliteTypeInfo, SqliteValueRef};
    use sqlx::ValueRef;
    use std::borrow::Cow;

    type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

    /// GEOS geometry wrapper for Geopackage geometry encoding/decoding
    pub struct Geometry<'a>(pub geos::Geometry<'a>);

    impl sqlx::Type<Sqlite> for Geometry<'_> {
        fn type_info() -> SqliteTypeInfo {
            <Vec<u8> as sqlx::Type<Sqlite>>::type_info()
        }
    }

    impl<'de> Decode<'de, Sqlite> for Geometry<'static> {
        fn decode(value: SqliteValueRef<'de>) -> Result<Self, BoxDynError> {
            if value.is_null() {
                // Return empty geometry
                return Ok(Geometry(geos::Geometry::create_empty_point().unwrap()));
            }
            let mut blob = <&[u8] as Decode<Sqlite>>::decode(value)?;
            let mut geo = Geos::new();
            wkb::process_gpkg_geom(&mut blob, &mut geo)
                .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
            let geom = Geometry(geo.geometry().to_owned());
            Ok(geom)
        }
    }

    impl<'q> Encode<'q, Sqlite> for Geometry<'_> {
        fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'q>>) -> IsNull {
            let mut wkb_out: Vec<u8> = Vec::new();
            let mut writer = wkb::WkbWriter::new(&mut wkb_out, wkb::WkbDialect::Geopackage);
            writer.dims.z = self.0.has_z().unwrap_or(false);
            writer.srid = self.0.get_srid().map(|srid| srid as i32).ok();
            process_geos(&self.0, &mut writer).expect("Failed to encode Geometry");

            args.push(SqliteArgumentValue::Blob(Cow::Owned(wkb_out)));

            IsNull::No
        }
    }
}
