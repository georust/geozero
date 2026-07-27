//! PostGIS geometry type encoding/decoding.
//!
//! All geometry types implementing [GeozeroGeometry](crate::GeozeroGeometry) can be encoded as PostGIS EWKB geometry using [wkb::Encode](crate::wkb::Encode).
//!
//! Geometry types implementing [FromWkb](crate::wkb::FromWkb) can be decoded from PostGIS geometries using [wkb::Decode](crate::wkb::Decode).
#[cfg(feature = "with-postgis-diesel")]
mod postgis_diesel;
#[cfg(feature = "with-postgis-postgres")]
mod postgis_postgres;
#[cfg(feature = "with-postgis-sqlx")]
mod postgis_sqlx;

/// PostGIS geometry type encoding/decoding for rust-postgres. Requires the `with-postgis-postgres` feature.
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
pub mod postgres {}

/// PostGIS geometry type encoding/decoding for SQLx. Requires the `with-postgis-sqlx` feature.
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
/// let geom: geo_types::Geometry<f64> = geo::Point::new(10.0, 20.0).into();
/// let _ = sqlx::query(
///     "INSERT INTO point2d (datetimefield,geom) VALUES(now(),ST_SetSRID($1,4326))",
/// )
/// .bind(wkb::Encode(geom))
/// .execute(&pool)
/// .await?;
/// # Ok(())
/// # }
/// ```
#[cfg(feature = "with-postgis-sqlx")]
pub mod sqlx {}

/// Postgis geometry type encoding for Diesel. Requires the `with-postgis-diesel` feature.
///
/// # PostGIS usage example with Diesel
///
/// Declare a table with a geometry column, and a model holding raw EWKB:
///
/// ```
/// use diesel::prelude::*;
/// use geozero::wkb::Ewkb;
///
/// diesel::table! {
///     use diesel::sql_types::*;
///     use geozero::postgis::diesel::sql_types::*;
///
///     geometries (name) {
///         name -> Varchar,
///         geom -> Nullable<Geometry>,
///     }
/// }
///
/// #[derive(Queryable, Debug, Insertable)]
/// #[diesel(table_name = geometries)]
/// pub struct Geom {
///     pub name: String,
///     pub geom: Option<Ewkb<Vec<u8>>>,
/// }
/// ```
///
/// A column can also be read into, and written from, any geometry type supported by GeoZero by
/// wrapping it in [`wkb::Decode`](crate::wkb::Decode) and [`wkb::Encode`](crate::wkb::Encode).
/// Below that is a postgis_diesel geometry, but `geo_types::Geometry<f64>` and the other supported
/// geometry types work the same way:
///
/// ```
/// use diesel::prelude::*;
/// use geozero::wkb;
/// use postgis_diesel::types::{GeometryContainer, Point};
///
/// # diesel::table! {
/// #     use diesel::sql_types::*;
/// #     use geozero::postgis::diesel::sql_types::*;
/// #
/// #     geometries (name) {
/// #         name -> Varchar,
/// #         geom -> Geometry,
/// #     }
/// # }
/// #
/// #[derive(Queryable, Debug)]
/// #[diesel(table_name = geometries)]
/// pub struct Geom {
///     pub name: String,
///     pub geom: wkb::Decode<GeometryContainer<Point>>,
/// }
///
/// #[derive(Insertable, Debug)]
/// #[diesel(table_name = geometries)]
/// pub struct NewGeom {
///     pub name: String,
///     pub geom: wkb::Encode<GeometryContainer<Point>>,
/// }
/// ```
///
/// A `Nullable<Geometry>` column needs an `Option<wkb::Decode<_>>`; the `geometry` field of
/// [`wkb::Decode`](crate::wkb::Decode) is always populated when decoding through Diesel.
///
/// The postgis_diesel geometry types implement [`GeozeroGeometry`](crate::GeozeroGeometry), so they
/// convert to any GeoZero-supported format:
///
/// ```
/// use geozero::ToWkt;
/// use postgis_diesel::types::Point;
///
/// let point = Point::new(10.0, 20.0, Some(4326));
/// assert_eq!(point.to_wkt().unwrap(), "POINT(10 20)");
/// ```
#[cfg(feature = "with-postgis-diesel")]
pub mod diesel {
    pub use super::postgis_diesel::*;
}
