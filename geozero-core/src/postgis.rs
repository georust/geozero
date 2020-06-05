/// Type conversions for [rust-postgres](https://github.com/sfackler/rust-postgres)
#[cfg(feature = "postgis-postgres")]
pub mod postgres {

    // This should be included in georust/geo to avoid a newtype
    /// PostGIS conversions for [georust/geo](https://github.com/georust/geo)
    pub mod geo {
        use crate::geo::RustGeo;
        use crate::wkb;
        use postgres_types::{FromSql, Type};

        pub struct Geometry(pub geo_types::Geometry<f64>);

        impl FromSql<'_> for Geometry {
            fn from_sql(
                _ty: &Type,
                raw: &[u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let mut rdr = std::io::Cursor::new(raw);
                let mut geo = RustGeo::new();
                wkb::process_ewkb_geom(&mut rdr, &mut geo)?;
                let geom = Geometry {
                    0: geo.geometry().to_owned(),
                };
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
        use crate::geos::Geos;
        use crate::wkb;
        use postgres_types::{FromSql, Type};

        pub struct Geometry<'a>(pub geos::Geometry<'a>);

        impl<'a> FromSql<'a> for Geometry<'a> {
            fn from_sql(
                _ty: &Type,
                raw: &[u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let mut rdr = std::io::Cursor::new(raw);
                let mut geo = Geos::new();
                wkb::process_ewkb_geom(&mut rdr, &mut geo)?;
                let geom = Geometry {
                    0: geo.geometry().to_owned(),
                };
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
        // #[ignore]
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
    /// PostGIS conversions for [georust/geo](https://github.com/georust/geo)
    pub mod geo {
        use crate::geo::RustGeo;
        use crate::wkb;
        use sqlx::decode::Decode;
        use sqlx::postgres::{PgData, PgTypeInfo, PgValue, Postgres};

        pub struct Geometry(pub geo_types::Geometry<f64>);

        impl sqlx::Type<Postgres> for Geometry {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("geometry")
            }
        }

        impl<'de> Decode<'de, Postgres> for Geometry {
            fn decode(value: PgValue<'de>) -> sqlx::Result<Self> {
                match value.get() {
                    Some(PgData::Binary(mut buf)) => {
                        let mut geo = RustGeo::new();
                        wkb::process_ewkb_geom(&mut buf, &mut geo)
                            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                        let geom = Geometry {
                            0: geo.geometry().to_owned(),
                        };
                        Ok(geom)
                    }
                    Some(PgData::Text(_s)) => Err(sqlx::Error::Decode(
                        "supporting binary geometry format only".into(),
                    )),
                    None => Ok(Geometry {
                        0: geo_types::Point::new(0., 0.).into(),
                    }),
                }
            }
        }
    }

    // This should be included in georust/geos to avoid a newtype
    /// PostGIS conversions for [GEOS](https://github.com/georust/geos)
    #[cfg(feature = "geos-lib")]
    pub mod geos {
        use crate::geos::Geos;
        use crate::wkb;
        use sqlx::decode::Decode;
        use sqlx::postgres::{PgData, PgTypeInfo, PgValue, Postgres};

        pub struct Geometry<'a>(pub geos::Geometry<'a>);

        impl sqlx::Type<Postgres> for Geometry<'_> {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("geometry")
            }
        }

        impl<'de> Decode<'de, Postgres> for Geometry<'static> {
            fn decode(value: PgValue<'de>) -> sqlx::Result<Self> {
                match value.get() {
                    Some(PgData::Binary(mut buf)) => {
                        let mut geo = Geos::new();
                        wkb::process_ewkb_geom(&mut buf, &mut geo)
                            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                        let geom = Geometry {
                            0: geo.geometry().to_owned(),
                        };
                        Ok(geom)
                    }
                    Some(PgData::Text(_s)) => Err(sqlx::Error::Decode(
                        "supporting binary geometry format only".into(),
                    )),
                    None => Ok(Geometry {
                        0: geos::Geometry::create_empty_point().unwrap(),
                    }),
                }
            }
        }
    }
}
