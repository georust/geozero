use geozero::wkb;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
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

#[tokio::test]
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

    let row: (wkb::GpkgWkb,) = sqlx::query_as("SELECT geom FROM pt2d")
        .fetch_one(&pool)
        .await?;

    let wkt = row.0.to_wkt().unwrap();
    assert_eq!(&wkt, "POINT(1.1 1.1)");

    Ok(())
}

#[tokio::test]
async fn rust_geo_query() -> Result<(), sqlx::Error> {
    use geozero::ToWkt;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://tests/data/gpkg_test.gpkg")
        .await?;

    let row: (wkb::Decode<geo_types::Geometry<f64>>,) = sqlx::query_as("SELECT geom FROM pt2d")
        .fetch_one(&pool)
        .await?;
    let geom = row.0.geometry.unwrap();
    println!("{}", geom.to_wkt().unwrap());
    assert_eq!(
        &format!("{geom:?}"),
        "Point(Point(Coord { x: 1.1, y: 1.1 }))"
    );

    Ok(())
}

#[tokio::test]
#[cfg(feature = "with-geos")]
async fn geos_query() -> Result<(), sqlx::Error> {
    use geos::Geom;

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect("sqlite://tests/data/gpkg_test.gpkg")
        .await?;

    let row: (wkb::Decode<geos::Geometry>,) = sqlx::query_as("SELECT geom FROM pt2d")
        .fetch_one(&pool)
        .await?;
    let geom = row.0.geometry.unwrap();
    assert_eq!(
        geom.to_wkt().unwrap(),
        "POINT (1.1000000000000001 1.1000000000000001)"
    );

    let row: (wkb::Decode<geos::Geometry>,) =
        sqlx::query_as("SELECT geom FROM pt2d WHERE geom IS NULL")
            .fetch_one(&pool)
            .await?;
    let value = row.0;
    assert!(value.geometry.is_none());

    // WKB encoding
    // let mut tx = pool.begin().await?;
    // let geom = geos::Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
    // // Requires loading an extension (e.g. SpatiaLite) providing functions like ST_SRID
    // let _inserted = sqlx::query("INSERT INTO pt2d (name,geom) VALUES('WKB Test',$1)")
    //     .bind(wkb::Encode(geom))
    //     .execute(&mut tx)
    //     .await?;
    // tx.commit().await?;

    Ok(())
}
