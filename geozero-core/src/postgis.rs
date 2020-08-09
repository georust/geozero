/// Type conversions for [rust-postgres](https://github.com/sfackler/rust-postgres)
#[cfg(feature = "postgis-postgres")]
pub mod postgres {

    // This should be included in georust/geo to avoid a newtype
    /// PostGIS conversions for [geo-types](https://github.com/georust/geo)
    pub mod geo {
        use crate::geo_types::Geo;
        use crate::wkb;
        use postgres_types::{FromSql, Type};

        pub struct Geometry(pub geo_types::Geometry<f64>);

        impl FromSql<'_> for Geometry {
            fn from_sql(
                _ty: &Type,
                raw: &[u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let mut rdr = std::io::Cursor::new(raw);
                let mut geo = Geo::new();
                wkb::process_ewkb_geom(&mut rdr, &mut geo)?;
                let geom = Geometry(geo.geometry().to_owned());
                Ok(geom)
            }

            fn accepts(ty: &Type) -> bool {
                match ty.name() {
                    "geography" | "geometry" => true,
                    _ => false,
                }
            }
        }

        #[test]
        #[ignore]
        fn geometry_query() -> Result<(), postgres::error::Error> {
            use postgres::{Client, NoTls};

            let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

            let row = client.query_one(
                "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
                &[],
            )?;

            let geom: Geometry = row.get(0);
            assert_eq!(&format!("{:?}", geom.0), "Polygon(Polygon { exterior: LineString([Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 2.0, y: 0.0 }, Coordinate { x: 2.0, y: 2.0 }, Coordinate { x: 0.0, y: 2.0 }, Coordinate { x: 0.0, y: 0.0 }]), interiors: [] })");
            Ok(())
        }
    }

    // This should be included in georust/geos to avoid a newtype
    /// PostGIS conversions for [GEOS](https://github.com/georust/geos)
    #[cfg(feature = "geos-lib")]
    pub mod geos {
        use crate::geos::{process_geos, Geos};
        use crate::wkb;
        use bytes::{buf::ext::BufMutExt, BytesMut};
        use geos::Geom;
        use postgres_types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
        use std::fmt;

        pub struct Geometry<'a>(pub geos::Geometry<'a>);

        impl<'a> FromSql<'a> for Geometry<'a> {
            fn from_sql(
                _ty: &Type,
                raw: &[u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let mut rdr = std::io::Cursor::new(raw);
                let mut geo = Geos::new();
                wkb::process_ewkb_geom(&mut rdr, &mut geo)?;
                let geom = Geometry(geo.geometry().to_owned());
                Ok(geom)
            }

            fn accepts(ty: &Type) -> bool {
                match ty.name() {
                    "geography" | "geometry" => true,
                    _ => false,
                }
            }
        }

        // required by ToSql
        impl<'a> fmt::Debug for Geometry<'a> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("geos::Geometry")
            }
        }

        impl<'a> ToSql for Geometry<'a> {
            fn to_sql(
                &self,
                _ty: &Type,
                out: &mut BytesMut,
            ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
                let pgout = &mut out.writer();
                let mut writer = wkb::WkbWriter::new(pgout, wkb::WkbDialect::Ewkb);
                writer.dims.z = self.0.has_z().unwrap_or(false);
                writer.srid = self.0.get_srid().map(|srid| srid as i32).ok();
                process_geos(&self.0, &mut writer)?;
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

        #[test]
        #[ignore]
        fn geometry_query() -> Result<(), postgres::error::Error> {
            use postgres::{Client, NoTls};

            let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

            let row = client.query_one(
                "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
                &[],
            )?;

            let geom: Geometry = row.get(0);
            assert_eq!(geom.0.to_wkt().unwrap(), "POLYGON ((0.0000000000000000 0.0000000000000000, 2.0000000000000000 0.0000000000000000, 2.0000000000000000 2.0000000000000000, 0.0000000000000000 2.0000000000000000, 0.0000000000000000 0.0000000000000000))");
            Ok(())
        }
    }
}

/// Type conversions for [SQLx](https://github.com/launchbadge/sqlx)
#[cfg(feature = "postgis-sqlx")]
pub mod sqlx {

    // This should be included in georust/geo to avoid a newtype
    /// PostGIS conversions for [geo-types](https://github.com/georust/geo)
    pub mod geo {
        use crate::geo_types::Geo;
        use crate::wkb;
        use sqlx::decode::Decode;
        use sqlx::postgres::{PgTypeInfo, PgValueRef, Postgres};
        use sqlx::ValueRef;

        type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

        pub struct Geometry(pub geo_types::Geometry<f64>);

        impl sqlx::Type<Postgres> for Geometry {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("geometry")
            }
        }

        impl<'de> Decode<'de, Postgres> for Geometry {
            fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
                if value.is_null() {
                    // Return empty geometry
                    return Ok(Geometry(geo_types::Geometry::GeometryCollection(
                        geo_types::GeometryCollection::new(),
                    )));
                }
                let mut blob = <&[u8] as Decode<Postgres>>::decode(value)?;
                let mut geo = Geo::new();
                wkb::process_ewkb_geom(&mut blob, &mut geo)
                    .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                let geom = Geometry(geo.geometry().to_owned());
                Ok(geom)
            }
        }
    }

    // This should be included in georust/geos to avoid a newtype
    /// PostGIS conversions for [GEOS](https://github.com/georust/geos)
    #[cfg(feature = "geos-lib")]
    pub mod geos {
        use crate::geos::{process_geos, Geos};
        use crate::wkb;
        use geos::Geom;
        use sqlx::decode::Decode;
        use sqlx::encode::{Encode, IsNull};
        use sqlx::postgres::{PgArgumentBuffer, PgTypeInfo, PgValueRef, Postgres};
        use sqlx::ValueRef;

        type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

        pub struct Geometry<'a>(pub geos::Geometry<'a>);

        impl sqlx::Type<Postgres> for Geometry<'_> {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("geometry")
            }
        }

        impl<'de> Decode<'de, Postgres> for Geometry<'static> {
            fn decode(value: PgValueRef<'de>) -> Result<Self, BoxDynError> {
                if value.is_null() {
                    // Return empty geometry
                    return Ok(Geometry(geos::Geometry::create_empty_point().unwrap()));
                }
                let mut blob = <&[u8] as Decode<Postgres>>::decode(value)?;
                let mut geo = Geos::new();
                wkb::process_ewkb_geom(&mut blob, &mut geo)
                    .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                let geom = Geometry(geo.geometry().to_owned());
                Ok(geom)
            }
        }

        impl Encode<'_, Postgres> for Geometry<'_> {
            fn encode_by_ref(&self, buf: &mut PgArgumentBuffer) -> IsNull {
                let mut wkb_out: Vec<u8> = Vec::new();
                let mut writer = wkb::WkbWriter::new(&mut wkb_out, wkb::WkbDialect::Ewkb);
                writer.dims.z = self.0.has_z().unwrap_or(false);
                writer.srid = self.0.get_srid().map(|srid| srid as i32).ok();
                process_geos(&self.0, &mut writer).expect("Failed to encode Geometry");
                buf.extend(&wkb_out); // Is there a way to write directly into PgArgumentBuffer?

                IsNull::No
            }
        }
    }
}
