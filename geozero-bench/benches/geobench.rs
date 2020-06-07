use criterion::{criterion_group, criterion_main, Criterion};
use geo::algorithm::centroid::Centroid;
use geo::Geometry as GeoGeometry;
use geozero::error::Result;
use geozero_core::geo::RustGeo;

mod fgb {
    use super::*;
    use flatgeobuf::*;
    use std::fs::File;
    use std::io::BufReader;

    pub(super) fn fgb_to_geo() -> Result<()> {
        let mut filein = BufReader::new(File::open("tests/data/countries.fgb")?);
        let mut fgb = FgbReader::open(&mut filein)?;
        fgb.select_all()?;
        let mut geo = RustGeo::new();
        fgb.process_features(&mut geo)?;
        // check last geometry
        if let GeoGeometry::MultiPolygon(mpoly) = geo.geometry() {
            assert_eq!(mpoly.centroid().unwrap().x(), -59.42097279311143);
        } else {
            assert!(false, "MultiPolygon expected");
        }
        Ok(())
    }

    pub(super) fn fgb_to_geo_bbox() -> Result<()> {
        let mut filein = BufReader::new(File::open("tests/data/countries.fgb")?);
        let mut fgb = FgbReader::open(&mut filein)?;
        fgb.select_bbox(8.8, 47.2, 9.5, 55.3)?;
        let mut geo = RustGeo::new();
        fgb.process_features(&mut geo)?;
        // check last geometry
        if let GeoGeometry::MultiPolygon(mpoly) = geo.geometry() {
            assert_eq!(mpoly.centroid().unwrap().x(), 96.87522333795941);
        } else {
            assert!(false, "MultiPolygon expected");
        }
        Ok(())
    }
}

mod postgis_postgres {
    use super::*;
    use postgres::{self, Client, NoTls};

    pub(super) fn postgis_postgres_to_geo() -> std::result::Result<(), postgres::error::Error> {
        use geozero_core::postgis::postgres::geo::Geometry as GeoZeroGeometry;

        // ogr2ogr -f PostgreSQL PG:dbname=testdb countries.fgb
        //
        // export DATABASE_URL=postgresql://pi@%2Fvar%2Frun%2Fpostgresql/testdb
        // export DATABASE_URL=postgresql://pi@localhost/testdb

        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        let mut geom = GeoZeroGeometry {
            0: geo_types::Point::new(0., 0.).into(),
        };
        for row in &client
            .query("SELECT geom FROM countries", &[])
            .unwrap()
        {
            geom = row.get(0);
        }
        // check last geometry
        if let GeoGeometry::MultiPolygon(mpoly) = geom.0 {
            assert_eq!(mpoly.centroid().unwrap().x(), -59.42097279311143);
        } else {
            assert!(false, "MultiPolygon expected");
        }
        Ok(())
    }
}

mod rust_postgis {
    // use geo::algorithm::from_postgis::FromPostgis;
    use postgis::ewkb;
    use postgres::{self, Client, NoTls};

    pub(super) fn rust_postgis_to_geo() -> std::result::Result<(), postgres::error::Error> {
        // ogr2ogr -f PostgreSQL PG:dbname=testdb countries.fgb
        //
        // export DATABASE_URL=postgresql://pi@%2Fvar%2Frun%2Fpostgresql/testdb
        // export DATABASE_URL=postgresql://pi@localhost/testdb

        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        for row in &client
            .query("SELECT geom FROM countries", &[])
            .unwrap()
        {
            let _pggeom: ewkb::MultiPolygon = row.get(0);
            // let geom = geo_types::MultiPolygon::from_postgis(&pggeom);
        }
        Ok(())
    }
}

mod postgis_sqlx {
    use super::*;
    use sqlx::postgres::PgConnection;
    use sqlx::prelude::*;
    use tokio::runtime::Runtime;

    async fn async_postgis_sqlx_to_geo() -> std::result::Result<(), sqlx::Error> {
        use geozero_core::postgis::sqlx::geo::Geometry as GeoZeroGeometry;

        // ogr2ogr -f PostgreSQL PG:dbname=testdb countries.fgb
        //
        // export DATABASE_URL=postgresql://pi@%2Fvar%2Frun%2Fpostgresql/testdb
        // export DATABASE_URL=postgresql://pi@localhost/testdb

        let mut conn = PgConnection::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

        let mut cursor = sqlx::query("SELECT geom FROM countries").fetch(&mut conn);
        let mut geom = GeoZeroGeometry {
            0: geo_types::Point::new(0., 0.).into(),
        };
        while let Some(row) = cursor.next().await? {
            geom = row.get::<GeoZeroGeometry, _>(0);
        }

        // check last geometry
        if let GeoGeometry::MultiPolygon(mpoly) = geom.0 {
            assert_eq!(mpoly.centroid().unwrap().x(), -59.42097279311143);
        } else {
            assert!(false, "MultiPolygon expected");
        }
        Ok(())
    }

    pub(super) fn postgis_sqlx_to_geo() {
        assert_eq!(
            Runtime::new()
                .unwrap()
                .block_on(async_postgis_sqlx_to_geo())
                .map_err(|e| e.to_string()),
            Ok(())
        );
    }
}

mod gpkg {
    use super::*;
    use sqlx::prelude::*;
    use sqlx::sqlite::SqliteConnection;
    use tokio::runtime::Runtime;

    async fn async_gpkg_to_geo() -> std::result::Result<(), sqlx::Error> {
        use geozero_core::gpkg::geo::Geometry as GeoZeroGeometry;

        // ogr2ogr -f GPKG countries.gpkg countries.fgb

        let mut conn = SqliteConnection::connect("sqlite://tests/data/countries.gpkg").await?;

        let mut cursor = sqlx::query("SELECT geom FROM countries").fetch(&mut conn);
        let mut geom = GeoZeroGeometry {
            0: geo_types::Point::new(0., 0.).into(),
        };
        while let Some(row) = cursor.next().await? {
            geom = row.get::<GeoZeroGeometry, _>(0);
        }

        // check last geometry
        if let GeoGeometry::MultiPolygon(mpoly) = geom.0 {
            assert_eq!(mpoly.centroid().unwrap().x(), -59.42097279311143);
        } else {
            assert!(false, "MultiPolygon expected");
        }
        Ok(())
    }

    pub(super) fn gpkg_to_geo() {
        assert_eq!(
            Runtime::new()
                .unwrap()
                .block_on(async_gpkg_to_geo())
                .map_err(|e| e.to_string()),
            Ok(())
        );
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("rust_postgis_to_geo", |b| {
        b.iter(|| rust_postgis::rust_postgis_to_geo())
    });
    c.bench_function("postgis_postgres_to_geo", |b| {
        b.iter(|| postgis_postgres::postgis_postgres_to_geo())
    });
    c.bench_function("postgis_sqlx_to_geo", |b| {
        b.iter(|| postgis_sqlx::postgis_sqlx_to_geo())
    });
    c.bench_function("fgb_to_geo", |b| b.iter(|| fgb::fgb_to_geo()));
    c.bench_function("fgb_to_geo_bbox", |b| b.iter(|| fgb::fgb_to_geo_bbox()));
    c.bench_function("gpkg_to_geo", |b| b.iter(|| gpkg::gpkg_to_geo()));
}

criterion_group!(name=benches; config=Criterion::default().sample_size(10); targets=criterion_benchmark);
criterion_main!(benches);
