mod pg {
    pub fn get_db_string() -> String {
        std::env::var("DATABASE_URL").unwrap()
    }

    #[cfg(feature = "with-postgis-sqlx")]
    pub async fn get_pool() -> sqlx::Pool<sqlx::Postgres> {
        sqlx::postgres::PgPoolOptions::new()
            .max_connections(5)
            .connect(&get_db_string())
            .await
            .unwrap()
    }
}

#[cfg(feature = "with-postgis-postgres")]
mod postgis_postgres {
    use crate::pg::get_db_string;
    use geozero::wkb;
    use geozero::wkt::WktWriter;
    use geozero::ToWkt as _;

    #[test]
    #[ignore]
    fn blob_query() -> Result<(), postgres::error::Error> {
        let mut client = postgres::Client::connect(&get_db_string(), postgres::NoTls).unwrap();
        let row = client.query_one(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry::bytea",
            &[],
        )?;
        let blob: &[u8] = row.get(0);
        let wkt = wkb::Ewkb(blob.to_vec()).to_wkt().expect("to_wkt failed");
        assert_eq!(&wkt, "POLYGON((0 0,2 0,2 2,0 2,0 0))");

        Ok(())
    }

    #[test]
    #[ignore]
    fn rust_geo_query() -> Result<(), postgres::error::Error> {
        let mut client = postgres::Client::connect(&get_db_string(), postgres::NoTls).unwrap();

        let row = client.query_one(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
            &[],
        )?;

        let value: wkb::Decode<geo_types::Geometry<f64>> = row.get(0);
        if let Some(geo_types::Geometry::Polygon(poly)) = value.geometry {
            assert_eq!(
                *poly.exterior(),
                vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0), (0.0, 0.0)].into()
            );
        } else {
            panic!("Conversion to geo_types::Geometry failed");
        }

        let row = client.query_one("SELECT NULL::geometry", &[])?;
        let value: wkb::Decode<geo_types::Geometry<f64>> = row.get(0);
        assert!(value.geometry.is_none());
        let row = client.query_one("SELECT NULL::geometry", &[])?;
        let value: Result<wkb::Decode<geo_types::Geometry<f64>>, _> = row.try_get(0);
        assert!(value.unwrap().geometry.is_none());

        // Insert geometry
        let geom: geo_types::Geometry<f64> = geo::Point::new(1.0, 3.0).into();
        let _ = client.execute(
            "INSERT INTO point2d (datetimefield,geom) VALUES(now(),ST_SetSRID($1,4326))",
            &[&wkb::Encode(geom)],
        );

        Ok(())
    }

    #[test]
    #[ignore]
    #[cfg(feature = "with-geos")]
    fn geos_query() -> Result<(), postgres::error::Error> {
        let mut client = postgres::Client::connect(&get_db_string(), postgres::NoTls).unwrap();

        let row = client.query_one(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
            &[],
        )?;

        let value: wkb::Decode<geos::Geometry> = row.get(0);
        assert_eq!(
            value.geometry.unwrap().to_wkt().unwrap(),
            "POLYGON((0 0,2 0,2 2,0 2,0 0))"
        );

        // Insert geometry
        let geom = geos::Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
        let _ = client.execute(
            "INSERT INTO point2d (datetimefield,geom) VALUES(now(),$1)",
            &[&wkb::Encode(geom)],
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
                matches!(ty.name(), "geography" | "geometry")
            }
        }

        #[test]
        #[ignore]
        fn geometry_query() -> Result<(), postgres::error::Error> {
            let mut client = postgres::Client::connect(&get_db_string(), postgres::NoTls).unwrap();

            let row = client.query_one(
                "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry",
                &[],
            )?;

            let wkt_geom: Wkt = row.get(0);
            assert_eq!(&wkt_geom.0, "POLYGON((0 0,2 0,2 2,0 2,0 0))");
            Ok(())
        }
    }
}

#[cfg(feature = "with-postgis-sqlx")]
mod postgis_sqlx {
    use super::PointZ;
    use crate::pg;
    use geozero::wkb;
    use geozero::ToWkt as _;

    #[tokio::test]
    #[ignore]
    async fn blob_query() -> Result<(), sqlx::Error> {
        let pool = pg::get_pool().await;

        let row: (Vec<u8>,) = sqlx::query_as(
            "SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry::bytea",
        )
        .fetch_one(&pool)
        .await?;

        let wkt = wkb::Ewkb(row.0).to_wkt().expect("to_wkt failed");
        assert_eq!(&wkt, "POLYGON((0 0,2 0,2 2,0 2,0 0))");

        let row: (wkb::Ewkb,) =
            sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                .fetch_one(&pool)
                .await?;

        let wkt = row.0.to_wkt().expect("to_wkt failed");
        assert_eq!(&wkt, "POLYGON((0 0,2 0,2 2,0 2,0 0))");

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn point3d_query() -> Result<(), sqlx::Error> {
        let pool = pg::get_pool().await;

        let row: (PointZ,) = sqlx::query_as("SELECT 'POINT(1 2 3)'::geometry")
            .fetch_one(&pool)
            .await?;

        let geom = row.0;
        assert_eq!(
            geom,
            PointZ {
                x: 1.0,
                y: 2.0,
                z: 3.0
            }
        );

        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn rust_geo_query() -> Result<(), sqlx::Error> {
        let pool = pg::get_pool().await;

        let row: (wkb::Decode<geo_types::Geometry<f64>>,) =
            sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                .fetch_one(&pool)
                .await?;

        let value = row.0;
        if let Some(geo_types::Geometry::Polygon(poly)) = value.geometry {
            assert_eq!(
                *poly.exterior(),
                vec![(0.0, 0.0), (2.0, 0.0), (2.0, 2.0), (0.0, 2.0), (0.0, 0.0)].into()
            );
        } else {
            panic!("Conversion to geo_types::Geometry failed");
        }

        let row: (wkb::Decode<geo_types::Geometry<f64>>,) = sqlx::query_as("SELECT NULL::geometry")
            .fetch_one(&pool)
            .await?;
        assert!(row.0.geometry.is_none());

        // Insert geometry
        let geom: geo_types::Geometry<f64> = geo::Point::new(10.0, 20.0).into();
        let inserted = sqlx::query(
            "INSERT INTO point2d (datetimefield, geom) VALUES(now(), ST_SetSRID($1,4326))",
        )
        .bind(wkb::Encode(geom))
        .execute(&pool)
        .await?;

        assert_eq!(inserted.rows_affected(), 1);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    async fn bulk_insert() -> Result<(), sqlx::Error> {
        // https://github.com/launchbadge/sqlx/blob/v0.5.13/FAQ.md#how-can-i-bind-an-array-to-a-values-clause-how-can-i-do-bulk-inserts
        let pool = pg::get_pool().await;

        let geom: geo_types::Geometry<f64> = geo::Point::new(10.0, 20.0).into();
        let geoms = vec![wkb::Encode(geom.clone()), wkb::Encode(geom.clone())];
        let inserted = sqlx::query(
            "INSERT INTO point2d (datetimefield, geom)
               SELECT now(), ST_SetSRID(g,4326) FROM UNNEST($1::geometry[]) as g",
        )
        .bind(&geoms[..])
        .execute(&pool)
        .await?;

        assert_eq!(inserted.rows_affected(), geoms.len() as u64);
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    // Requires DATABASE_URL at compile time
    #[cfg(feature = "dont-compile")]
    async fn rust_geo_macro_query() -> Result<(), sqlx::Error> {
        let pool = pg::get_pool().await;

        let mut tx = pool.begin().await?;

        let _ = sqlx::query!("DELETE FROM point2d").execute(&mut tx).await?;

        let rec = sqlx::query!("SELECT count(*) as count FROM point2d")
            .fetch_one(&mut tx)
            .await?;
        assert_eq!(rec.count, Some(0));

        let geom: geo_types::Geometry<f64> = geo::Point::new(10.0, 20.0).into();
        // https://docs.rs/sqlx/0.5.1/sqlx/macro.query.html?search=insert#type-overrides-bind-parameters-postgres-only
        let inserted = sqlx::query!(
            "INSERT INTO point2d (datetimefield, geom) VALUES(now(), ST_SetSRID($1::geometry,4326))",
            wkb::Encode(geom) as _
        )
        .execute(&mut tx)
        .await?;

        assert_eq!(inserted.rows_affected(), 1);

        // https://docs.rs/sqlx/0.5.1/sqlx/macro.query.html#force-a-differentcustom-type
        let rec = sqlx::query!(
            r#"SELECT datetimefield, geom as "geom!: wkb::Decode<geo_types::Geometry<f64>>" FROM point2d"#
        )
        .fetch_one(&mut tx)
        .await?;
        assert_eq!(
            rec.geom.geometry.unwrap(),
            geo::Point::new(10.0, 20.0).into()
        );

        struct PointRec {
            pub geom: wkb::Decode<geo_types::Geometry<f64>>,
            pub datetimefield: Option<sqlx::types::time::OffsetDateTime>,
        }
        let rec = sqlx::query_as!(
            PointRec,
            r#"SELECT datetimefield, geom as "geom!: _" FROM point2d"#
        )
        .fetch_one(&mut tx)
        .await?;
        assert_eq!(
            rec.geom.geometry.unwrap(),
            geo::Point::new(10.0, 20.0).into()
        );

        tx.rollback().await?;
        Ok(())
    }

    #[tokio::test]
    #[ignore]
    #[cfg(feature = "with-geos")]
    async fn geos_query() -> Result<(), sqlx::Error> {
        let pool = pg::get_pool().await;

        let row: (wkb::Decode<geos::Geometry>,) =
            sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                .fetch_one(&pool)
                .await?;
        let value = row.0;
        assert_eq!(
            value.geometry.unwrap().to_wkt().unwrap(),
            "POLYGON((0 0,2 0,2 2,0 2,0 0))"
        );

        let row: (wkb::Decode<geos::Geometry>,) = sqlx::query_as("SELECT NULL::geometry")
            .fetch_one(&pool)
            .await?;
        let value = row.0;
        assert!(value.geometry.is_none());

        // Insert geometry
        let geom = geos::Geometry::new_from_wkt("POINT(1 3)").expect("Invalid geometry");
        let inserted = sqlx::query("INSERT INTO point2d (datetimefield,geom) VALUES(now(),$1)")
            .bind(wkb::Encode(geom))
            .execute(&pool)
            .await?;

        assert_eq!(inserted.rows_affected(), 1);

        Ok(())
    }

    mod register_type {
        use super::*;
        use geozero::wkt::WktWriter;
        use sqlx::decode::Decode;
        use sqlx::postgres::{PgTypeInfo, PgValueRef, Postgres};
        use sqlx::ValueRef;

        type BoxDynError = Box<dyn std::error::Error + Send + Sync>;

        struct Text(String);

        impl sqlx::Type<Postgres> for Text {
            fn type_info() -> PgTypeInfo {
                PgTypeInfo::with_name("geometry")
            }
        }

        impl<'de> Decode<'de, Postgres> for Text {
            fn decode(value: PgValueRef) -> Result<Self, BoxDynError> {
                if value.is_null() {
                    return Ok(Text("EMPTY".to_string()));
                }
                let mut blob = <&[u8] as Decode<Postgres>>::decode(value)?;
                let mut data: Vec<u8> = Vec::new();
                let mut writer = WktWriter::new(&mut data);
                wkb::process_ewkb_geom(&mut blob, &mut writer)
                    .map_err(|e| sqlx::Error::Decode(e.to_string().into()))?;
                let text = Text(std::str::from_utf8(&data).unwrap().to_string());
                Ok(text)
            }
        }

        #[tokio::test]
        #[ignore]
        async fn geometry_query() -> Result<(), sqlx::Error> {
            let pool = pg::get_pool().await;

            let row: (Text,) =
                sqlx::query_as("SELECT 'SRID=4326;POLYGON ((0 0, 2 0, 2 2, 0 2, 0 0))'::geometry")
                    .fetch_one(&pool)
                    .await?;
            assert_eq!((row.0).0, "POLYGON((0 0,2 0,2 2,0 2,0 0))");

            let row: (Text,) = sqlx::query_as("SELECT NULL::geometry")
                .fetch_one(&pool)
                .await?;
            assert_eq!((row.0).0, "EMPTY");

            Ok(())
        }
    }
}

// --- Minimal geometry implementation with PostGIS/GPKG support

use geozero::wkb::{FromWkb, WkbDialect};
use geozero::{CoordDimensions, GeomProcessor, GeozeroGeometry};
use std::io::Read;

#[derive(Debug, PartialEq, Default)]
struct PointZ {
    x: f64,
    y: f64,
    z: f64,
}

impl GeomProcessor for PointZ {
    fn dimensions(&self) -> CoordDimensions {
        CoordDimensions::xyz()
    }
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) -> geozero::error::Result<()> {
        self.x = x;
        self.y = y;
        self.z = z.unwrap_or(0.0);
        Ok(())
    }
}

impl GeozeroGeometry for PointZ {
    fn process_geom<P: GeomProcessor>(
        &self,
        processor: &mut P,
    ) -> Result<(), geozero::error::GeozeroError> {
        processor.point_begin(0)?;
        processor.coordinate(self.x, self.y, Some(self.z), None, None, None, 0)?;
        processor.point_end(0)
    }
    fn dims(&self) -> CoordDimensions {
        CoordDimensions::xyz()
    }
}

impl FromWkb for PointZ {
    fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> geozero::error::Result<Self> {
        let mut pt = PointZ::default();
        geozero::wkb::process_wkb_type_geom(rdr, &mut pt, dialect)?;
        Ok(pt)
    }
}

#[cfg(feature = "with-postgis-postgres")]
mod postgis_postgres_macros {
    geozero::impl_postgres_postgis_decode!(super::PointZ);
    geozero::impl_postgres_postgis_encode!(super::PointZ);
}
#[cfg(feature = "with-postgis-sqlx")]
mod postgis_sqlx_macros {
    geozero::impl_sqlx_postgis_type_info!(super::PointZ);
    geozero::impl_sqlx_postgis_decode!(super::PointZ);
    geozero::impl_sqlx_postgis_encode!(super::PointZ);
}
#[cfg(feature = "with-gpkg")]
mod gpkg_sqlx_macros {
    geozero::impl_sqlx_gpkg_type_info!(super::PointZ);
    geozero::impl_sqlx_gpkg_decode!(super::PointZ);
    geozero::impl_sqlx_gpkg_encode!(super::PointZ);
}
