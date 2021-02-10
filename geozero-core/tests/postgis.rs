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
        assert!(wkb::process_ewkb_geom(&mut geom, &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "POLYGON((0 0,2 0,2 2,0 2,0 0))"
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
        if let geo_types::Geometry::Polygon(poly) = geom.0 {
            assert_eq!(&format!("{:?}", poly), "Polygon { exterior: LineString([Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 2.0, y: 0.0 }, Coordinate { x: 2.0, y: 2.0 }, Coordinate { x: 0.0, y: 2.0 }, Coordinate { x: 0.0, y: 0.0 }]), interiors: [] }");
        } else {
            assert!(false, "Conversion to geo_types::Geometry failed");
        }

        // WKB encoding
        let geom = geo::Point::new(1.0, 3.0).into();
        let _ = client.execute(
            "INSERT INTO point2d (datetimefield,geom) VALUES(now(),$1)",
            &[&Geometry(geom)],
        );

        Ok(())
    }

    #[test]
    #[ignore]
    #[cfg(feature = "geos-lib")]
    fn geos_query() -> Result<(), postgres::error::Error> {
        use geos::Geom;
        use geozero_core::postgis::postgres::geos::Geometry;

        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        let row = client.query_one(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
            &[],
        )?;

        let geom: Geometry = row.get(0);
        assert_eq!(geom.0.to_wkt().unwrap(), "POLYGON ((0.0000000000000000 0.0000000000000000, 2.0000000000000000 0.0000000000000000, 2.0000000000000000 2.0000000000000000, 0.0000000000000000 2.0000000000000000, 0.0000000000000000 0.0000000000000000))");

        // WKB encoding
        let geom = geos::Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
        let _ = client.execute(
            "INSERT INTO point2d (datetimefield,geom) VALUES(now(),$1)",
            &[&Geometry(geom)],
        );

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
                wkb::process_ewkb_geom(&mut rdr, &mut writer)?;
                let wkt = Wkt(std::str::from_utf8(&wkt_data)?.to_string());
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
            assert_eq!(&geom.0, "POLYGON((0 0,2 0,2 2,0 2,0 0))");
            Ok(())
        }
    }
}

#[cfg(feature = "postgis-sqlx")]
mod postgis_sqlx {
    use geozero_core::wkb;
    use geozero_core::wkt::WktWriter;
    use sqlx::postgres::PgPoolOptions;
    use std::env;
    use tokio::runtime::Runtime;

    async fn blob_query() -> Result<(), sqlx::Error> {
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var("DATABASE_URL").unwrap())
            .await?;

        let row: (Vec<u8>,) = sqlx::query_as(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry::bytea",
        )
        .fetch_one(&pool)
        .await?;

        let mut wkt_data: Vec<u8> = Vec::new();
        let mut writer = WktWriter::new(&mut wkt_data);
        assert!(wkb::process_ewkb_geom(&mut row.0.as_slice(), &mut writer).is_ok());
        assert_eq!(
            std::str::from_utf8(&wkt_data).unwrap(),
            "POLYGON((0 0,2 0,2 2,0 2,0 0))"
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
        use sqlx::Done;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var("DATABASE_URL").unwrap())
            .await?;

        let row: (Geometry,) =
            sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                .fetch_one(&pool)
                .await?;

        let geom = row.0;
        if let geo_types::Geometry::Polygon(poly) = geom.0 {
            assert_eq!(&format!("{:?}", poly), "Polygon { exterior: LineString([Coordinate { x: 0.0, y: 0.0 }, Coordinate { x: 2.0, y: 0.0 }, Coordinate { x: 2.0, y: 2.0 }, Coordinate { x: 0.0, y: 2.0 }, Coordinate { x: 0.0, y: 0.0 }]), interiors: [] }");
        } else {
            assert!(false, "Conversion to geo_types::Geometry failed");
        }

        let row: (Geometry,) = sqlx::query_as("SELECT NULL::geometry")
            .fetch_one(&pool)
            .await?;

        assert_eq!(
            &format!("{:?}", (row.0).0),
            "GeometryCollection(GeometryCollection([]))"
        );

        // WKB encoding
        let mut tx = pool.begin().await?;
        let geom = geo::Point::new(10.0, 20.0).into();
        let inserted = sqlx::query("INSERT INTO point2d (datetimefield,geom) VALUES(now(),$1)")
            .bind(Geometry(geom))
            .execute(&mut tx)
            .await?;
        tx.commit().await?;

        assert_eq!(inserted.rows_affected(), 1);

        Ok(())
    }

    #[test]
    #[ignore]
    fn async_rust_geo_query() {
        assert!(Runtime::new().unwrap().block_on(rust_geo_query()).is_ok());
    }

    #[cfg(feature = "geos-lib")]
    async fn geos_query() -> Result<(), sqlx::Error> {
        use geos::Geom;
        use geozero_core::postgis::sqlx::geos::Geometry;

        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&env::var("DATABASE_URL").unwrap())
            .await?;

        let row: (Geometry,) =
            sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                .fetch_one(&pool)
                .await?;
        let geom = row.0;
        assert_eq!(geom.0.to_wkt().unwrap(), "POLYGON ((0.0000000000000000 0.0000000000000000, 2.0000000000000000 0.0000000000000000, 2.0000000000000000 2.0000000000000000, 0.0000000000000000 2.0000000000000000, 0.0000000000000000 0.0000000000000000))");

        let row: (Geometry,) = sqlx::query_as("SELECT NULL::geometry")
            .fetch_one(&pool)
            .await?;
        let geom = row.0;
        assert_eq!(geom.0.to_wkt().unwrap(), "POINT EMPTY");

        // WKB encoding
        let mut tx = pool.begin().await?;
        let geom = geos::Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
        let _inserted = sqlx::query("INSERT INTO point2d (datetimefield,geom) VALUES(now(),$1)")
            .bind(Geometry(geom))
            .execute(&mut tx)
            .await?;
        tx.commit().await?;

        Ok(())
    }

    #[test]
    #[ignore]
    #[cfg(feature = "geos-lib")]
    fn async_geos_query() {
        assert_eq!(
            Runtime::new()
                .unwrap()
                .block_on(geos_query())
                .map_err(|e| e.to_string()),
            Ok(())
        );
    }

    mod register_type {
        use super::*;
        use sqlx::decode::Decode;
        use sqlx::postgres::{PgTypeInfo, PgValueRef, Postgres};
        use sqlx::ValueRef;

        type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

        struct Wkt(String);

        impl sqlx::Type<Postgres> for Wkt {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("geometry")
            }
        }

        impl<'de> Decode<'de, Postgres> for Wkt {
            fn decode(value: PgValueRef) -> Result<Self, BoxDynError> {
                if value.is_null() {
                    return Ok(Wkt("EMPTY".to_string()));
                }
                let mut blob = <&[u8] as Decode<Postgres>>::decode(value)?;
                let mut wkt_data: Vec<u8> = Vec::new();
                let mut writer = WktWriter::new(&mut wkt_data);
                wkb::process_ewkb_geom(&mut blob, &mut writer)
                    .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                let wkt = Wkt(std::str::from_utf8(&wkt_data).unwrap().to_string());
                Ok(wkt)
            }
        }

        async fn geometry_query() -> Result<(), sqlx::Error> {
            let pool = PgPoolOptions::new()
                .max_connections(5)
                .connect(&env::var("DATABASE_URL").unwrap())
                .await?;

            let row: (Wkt,) =
                sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                    .fetch_one(&pool)
                    .await?;
            assert_eq!((row.0).0, "POLYGON((0 0,2 0,2 2,0 2,0 0))");

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
