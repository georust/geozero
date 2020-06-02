#[cfg(feature = "postgis-postgres")]
mod postgis_postgres {
    use geozero_core::wkb;
    use geozero_core::wkt::WktWriter;
    use postgres::{Client, NoTls};

    #[test]
    #[ignore]
    fn blob_query() -> Result<(), postgres::error::Error> {
        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        let row = client.query_one(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry::bytea",
            &[],
        )?;
        let mut geom: &[u8] = row.get(0);
        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        assert!(wkb::process_wkb_geom(&mut geom, &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))"
        );

        Ok(())
    }

    #[test]
    #[ignore]
    fn rust_geo_query() -> Result<(), postgres::error::Error> {
        use geozero_core::postgis::postgres::geo::Geometry;

        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        let row = client.query_one(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
            &[],
        )?;

        let geom: Geometry = row.get(0);
        assert_eq!(&format!("{:?}", geom.0), "Polygon(Polygon { exterior: LineString([Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 2.0, y: 0.0 }, Coordinate { x: 2.0, y: 2.0 }, Coordinate { x: 0.0, y: 2.0 }, Coordinate { x: 0.0, y: 0.0 }]), interiors: [] })");
        Ok(())
    }

    mod register_type {
        use super::*;
        use postgres_types::{FromSql, Type};

        struct Wkt(String);

        impl FromSql<'_> for Wkt {
            fn from_sql(
                _ty: &Type,
                raw: &[u8],
            ) -> Result<Self, Box<dyn std::error::Error + Sync + Send>> {
                let mut rdr = std::io::Cursor::new(raw);
                let mut wkt_data: Vec<u8> = Vec::new();
                let mut writer = WktWriter::new(&mut wkt_data);
                wkb::process_wkb_geom(&mut rdr, &mut writer)?;
                let wkt = Wkt {
                    0: std::str::from_utf8(&wkt_data)?.to_string(),
                };
                Ok(wkt)
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
            let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

            let row = client.query_one(
                "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
                &[],
            )?;

            let geom: Wkt = row.get(0);
            assert_eq!(&geom.0, "POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))");
            Ok(())
        }
    }
}

#[cfg(feature = "postgis-sqlx")]
mod postgis_sqlx {
    use geozero_core::wkb;
    use geozero_core::wkt::WktWriter;
    use sqlx::postgres::{PgPool, PgQueryAs};
    use tokio::runtime::Runtime;

    async fn blob_query() -> Result<(), sqlx::Error> {
        let pool = PgPool::builder()
            .max_size(5)
            .build(&std::env::var("DATABASE_URL").unwrap())
            .await?;

        let row: (Vec<u8>,) = sqlx::query_as(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry::bytea",
        )
        .fetch_one(&pool)
        .await?;

        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        assert!(wkb::process_wkb_geom(&mut row.0.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))"
        );

        Ok(())
    }

    #[test]
    #[ignore]
    fn async_blob_query() {
        assert!(Runtime::new().unwrap().block_on(blob_query()).is_ok());
    }

    async fn rust_geo_query() -> Result<(), sqlx::Error> {
        use geozero_core::postgis::sqlx::geo::Geometry;

        let pool = PgPool::builder()
            .max_size(5)
            .build(&std::env::var("DATABASE_URL").unwrap())
            .await?;

        let row: (Geometry,) =
            sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                .fetch_one(&pool)
                .await?;

        assert_eq!(&format!("{:?}", (row.0).0), "Polygon(Polygon { exterior: LineString([Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 2.0, y: 0.0 }, Coordinate { x: 2.0, y: 2.0 }, Coordinate { x: 0.0, y: 2.0 }, Coordinate { x: 0.0, y: 0.0 }]), interiors: [] })");
        Ok(())
    }

    #[test]
    #[ignore]
    fn async_rust_geo_query() {
        assert!(Runtime::new().unwrap().block_on(rust_geo_query()).is_ok());
    }

    mod register_type {
        use super::*;
        use sqlx::decode::Decode;
        use sqlx::postgres::{PgData, PgTypeInfo, PgValue, Postgres};

        struct Wkt(String);

        impl sqlx::Type<Postgres> for Wkt {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("geometry")
            }
        }

        impl<'de> Decode<'de, Postgres> for Wkt {
            fn decode(value: PgValue<'de>) -> sqlx::Result<Self> {
                match value.get() {
                    Some(PgData::Binary(mut buf)) => {
                        let mut wkt_data: Vec<u8> = Vec::new();
                        let mut writer = WktWriter::new(&mut wkt_data);
                        wkb::process_wkb_geom(&mut buf, &mut writer)
                            .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                        let wkt = Wkt {
                            0: std::str::from_utf8(&wkt_data).unwrap().to_string(),
                        };
                        Ok(wkt)
                    }
                    Some(PgData::Text(_s)) => Err(sqlx::Error::Decode(
                        "supporting binary geometry format only".into(),
                    )),
                    None => Ok(Wkt {
                        0: "EMPTY".to_string(),
                    }),
                }
            }
        }

        async fn geometry_query() -> Result<(), sqlx::Error> {
            let pool = PgPool::builder()
                .max_size(5)
                .build(&std::env::var("DATABASE_URL").unwrap())
                .await?;

            let row: (Wkt,) =
                sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                    .fetch_one(&pool)
                    .await?;
            assert_eq!((row.0).0, "POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))");

            let row: (Wkt,) = sqlx::query_as("SELECT NULL::geometry")
                .fetch_one(&pool)
                .await?;
            assert_eq!((row.0).0, "EMPTY");

            Ok(())
        }

        #[test]
        #[ignore]
        fn postgis_geometry_query() {
            assert!(Runtime::new().unwrap().block_on(geometry_query()).is_ok());
        }
    }
}
