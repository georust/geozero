//! PostGIS geometry type encoding/decoding.
//!
//! All geometry types implementing [GeozeroGeometry](crate::GeozeroGeometry) can be encoded as PostGIS EWKB geometry.
//!
//! Geometry types implementing [FromWkb](crate::wkb::FromWkb) can be decoded from PostGIS geometries.
#[cfg(feature = "with-postgis-postgres")]
mod postgis_postgres;
#[cfg(feature = "with-postgis-sqlx")]
mod postgis_sqlx;

/// PostGIS geometry type encoding/decoding for rust-postgres.
///
/// # PostGIS usage example with rust-postgres
///
/// Select and insert geo-types geometries:
/// ```
/// use geozero::wkb;
/// use postgres::{Client, NoTls};
///
/// # fn rust_geo_query() -> Result<(), postgres::error::Error> {
/// let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;
///
/// let row = client.query_one(
///     "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
///     &[],
/// )?;
///
/// let value: wkb::Decode<geo_types::Geometry<f64>> = row.get(0);
/// if let Some(geo_types::Geometry::Polygon(poly)) = value.geometry {
///     assert_eq!(
///         *poly.exterior(),
///         vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0), (0.0, 0.0)].into()
///     );
/// }
///
/// // Insert geometry
/// let geom: geo_types::Geometry<f64> = geo::Point::new(1.0, 3.0).into();
/// let _ = client.execute(
///     "INSERT INTO point2d (datetimefield,geom) VALUES(now(),ST_SetSRID($1,4326))",
///     &[&wkb::Encode(geom)],
/// );
/// # Ok(())
/// # }
///```
#[cfg(feature = "with-postgis-postgres")]
pub mod postgres {
    pub use super::postgis_postgres::*;
}
/// PostGIS geometry type encoding/decoding for SQLx.
///
/// # PostGIS usage example with SQLx
///
/// Select and insert geo-types geometries with SQLx:
/// ```
/// use geozero::wkb;
/// use sqlx::postgres::PgPoolOptions;
/// # use std::env;
///
/// # async fn rust_geo_query() -> Result<(), sqlx::Error> {
/// let pool = PgPoolOptions::new()
///     .max_connections(5)
///     .connect(&env::var("DATABASE_URL").unwrap())
///     .await?;
///
/// let row: (wkb::Decode<geo_types::Geometry<f64>>,) =
///     sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
///         .fetch_one(&pool)
///         .await?;
/// let value = row.0;
/// if let Some(geo_types::Geometry::Polygon(poly)) = value.geometry {
///     assert_eq!(
///         *poly.exterior(),
///         vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0), (0.0, 0.0)].into()
///     );
/// }
///
/// // Insert geometry
/// let mut tx = pool.begin().await?;
/// let geom: geo_types::Geometry<f64> = geo::Point::new(10.0, 20.0).into();
/// let _ = sqlx::query(
///     "INSERT INTO point2d (datetimefield,geom) VALUES(now(),ST_SetSRID($1,4326))",
/// )
/// .bind(wkb::Encode(geom))
/// .execute(&mut tx)
/// .await?;
/// tx.commit().await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "with-postgis-sqlx")]
pub mod sqlx {
    pub use super::postgis_sqlx::*;
}
