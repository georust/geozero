#[cfg(feature = "gpkg")]
mod gpkg_sqlx {
    use geozero::wkb;
    use sqlx::sqlite::SqlitePoolOptions;
    use tokio::runtime::Runtime;

    async fn geometry_columns_query() -> Result<(), sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite://tests/data/gpkg_test.gpkg")
            .await?;

        let row: (String,String,) = sqlx::query_as(
            "SELECT table_name, column_name, geometry_type_name, srs_id, z, m FROM gpkg_geometry_columns"
            )
            .fetch_one(&pool)
            .await?;

        dbg!(&row);
        assert_eq!(row, ("pt2d".to_string(), "geom".to_string()));

        Ok(())
    }

    #[test]
    fn async_geometry_columns_query() {
        assert_eq!(
            Runtime::new()
                .unwrap()
                .block_on(geometry_columns_query())
                .map_err(|e| e.to_string()),
            Ok(())
        );
    }

    async fn blob_query() -> Result<(), sqlx::Error> {
        use geozero::ToWkt;

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite://tests/data/gpkg_test.gpkg")
            .await?;

        let row: (Vec<u8>,) = sqlx::query_as("SELECT geom FROM pt2d")
            .fetch_one(&pool)
            .await?;

        let wkt = wkb::GpkgWkb(row.0).to_wkt().unwrap();
        assert_eq!(&wkt, "POINT(1.1 1.1)");

        Ok(())
    }

    #[test]
    fn async_blob_query() {
        assert_eq!(
            Runtime::new()
                .unwrap()
                .block_on(blob_query())
                .map_err(|e| e.to_string()),
            Ok(())
        );
    }

    async fn rust_geo_query() -> Result<(), sqlx::Error> {
        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite://tests/data/gpkg_test.gpkg")
            .await?;

        let row: (wkb::Geometry<geo_types::Geometry<f64>>,) =
            sqlx::query_as("SELECT geom FROM pt2d")
                .fetch_one(&pool)
                .await?;
        let geom = row.0;
        assert_eq!(
            &format!("{:?}", geom.0),
            "Point(Point(Coordinate { x: 1.1, y: 1.1 }))"
        );

        Ok(())
    }

    #[test]
    fn async_rust_geo_query() {
        assert_eq!(
            Runtime::new()
                .unwrap()
                .block_on(rust_geo_query())
                .map_err(|e| e.to_string()),
            Ok(())
        );
    }

    #[cfg(feature = "geos-lib")]
    async fn geos_query() -> Result<(), sqlx::Error> {
        use geos::Geom;

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect("sqlite://tests/data/gpkg_test.gpkg")
            .await?;

        let row: (wkb::Geometry<geos::Geometry>,) = sqlx::query_as("SELECT geom FROM pt2d")
            .fetch_one(&pool)
            .await?;
        let geom = row.0;
        assert_eq!(
            geom.0.to_wkt().unwrap(),
            "POINT (1.1000000000000001 1.1000000000000001)"
        );

        let row: (wkb::Geometry<geos::Geometry>,) =
            sqlx::query_as("SELECT geom FROM pt2d WHERE geom IS NULL")
                .fetch_one(&pool)
                .await?;
        let geom = row.0;
        assert_eq!(geom.0.to_wkt().unwrap(), "POINT EMPTY");

        // WKB encoding
        // let mut tx = pool.begin().await?;
        // let geom = geos::Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
        // // Requires loading an extension (e.g. SpatiaLite) providing functions like ST_SRID
        // let _inserted = sqlx::query("INSERT INTO pt2d (name,geom) VALUES('WKB Test',$1)")
        //     .bind(wkb::Geometry(geom))
        //     .execute(&mut tx)
        //     .await?;
        // tx.commit().await?;

        Ok(())
    }

    #[test]
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
}
