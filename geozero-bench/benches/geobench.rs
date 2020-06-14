use criterion::{criterion_group, criterion_main, Criterion};
use geozero::error::Result;
use geozero_core::geo::RustGeo;

mod fgb {
    use super::*;
    use flatgeobuf::*;
    use std::fs::File;
    use std::io::BufReader;

    pub(super) fn fgb_to_geo(fpath: &str, count: usize) -> Result<()> {
        let mut filein = BufReader::new(File::open(fpath)?);
        let mut fgb = FgbReader::open(&mut filein)?;
        fgb.select_all()?;
        let mut geo = RustGeo::new();
        fgb.process_features(&mut geo)?;
        assert_eq!(fgb.features_count(), count);
        Ok(())
    }

    pub(super) fn fgb_to_geo_bbox(
        fpath: &str,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        count: usize,
    ) -> Result<()> {
        let mut filein = BufReader::new(File::open(fpath)?);
        let mut fgb = FgbReader::open(&mut filein)?;
        fgb.select_bbox(min_x, min_y, max_x, max_y)?;
        let mut geo = RustGeo::new();
        fgb.process_features(&mut geo)?;
        assert_eq!(fgb.features_count(), count);
        Ok(())
    }
}

mod postgis_postgres {
    use geozero_core::postgis::postgres::geo::Geometry as GeoZeroGeometry;
    use postgres::{self, Client, NoTls};

    // export DATABASE_URL=postgresql://pi@%2Fvar%2Frun%2Fpostgresql/testdb
    // export DATABASE_URL=postgresql://pi@localhost/testdb

    pub(super) fn postgis_postgres_to_geo(
        table: &str,
        count: usize,
    ) -> std::result::Result<(), postgres::error::Error> {
        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        let mut cnt = 0;
        for row in &client
            .query(format!("SELECT geom FROM {}", table).as_str(), &[])
            .unwrap()
        {
            let _geom: GeoZeroGeometry = row.get(0);
            cnt += 1;
        }
        assert_eq!(cnt, count);
        Ok(())
    }

    pub(super) fn postgis_postgres_to_geo_bbox(
        table: &str,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        srid: i32,
        count: usize,
    ) -> std::result::Result<(), postgres::error::Error> {
        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        let mut cnt = 0;
        let sql = format!(
            "SELECT geom FROM {} WHERE geom && ST_MakeEnvelope({}, {}, {}, {}, {})",
            table, min_x, min_y, max_x, max_y, srid
        );
        for row in &client.query(sql.as_str(), &[]).unwrap() {
            let _geom: GeoZeroGeometry = row.get(0);
            cnt += 1;
        }
        assert_eq!(cnt, count);
        Ok(())
    }
}

mod rust_postgis {
    // use geo::algorithm::from_postgis::FromPostgis;
    use postgis::ewkb;
    use postgres::{self, Client, NoTls};

    // export DATABASE_URL=postgresql://pi@%2Fvar%2Frun%2Fpostgresql/testdb
    // export DATABASE_URL=postgresql://pi@localhost/testdb

    pub(super) fn rust_postgis_to_geo(
        table: &str,
        count: usize,
    ) -> std::result::Result<(), postgres::error::Error> {
        let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

        let mut cnt = 0;
        for row in &client
            .query(format!("SELECT geom FROM {}", table).as_str(), &[])
            .unwrap()
        {
            let _pggeom: ewkb::MultiPolygon = row.get(0);
            // let geom = geo_types::MultiPolygon::from_postgis(&pggeom);
            cnt += 1;
        }
        assert_eq!(cnt, count);
        Ok(())
    }
}

mod postgis_sqlx {
    use futures_util::stream::TryStreamExt;
    use sqlx::postgres::PgConnection;
    use sqlx::prelude::*;

    // export DATABASE_URL=postgresql://pi@%2Fvar%2Frun%2Fpostgresql/testdb
    // export DATABASE_URL=postgresql://pi@localhost/testdb

    pub(super) async fn postgis_sqlx_to_geo(
        table: &str,
        count: usize,
    ) -> std::result::Result<(), sqlx::Error> {
        use geozero_core::postgis::sqlx::geo::Geometry as GeoZeroGeometry;

        let mut conn = PgConnection::connect(&std::env::var("DATABASE_URL").unwrap()).await?;

        let sql = format!("SELECT geom FROM {}", table);
        let mut cursor = sqlx::query(&sql).fetch(&mut conn);

        let mut cnt = 0;
        while let Some(row) = cursor.try_next().await? {
            let _geom = row.get::<GeoZeroGeometry, _>(0);
            cnt += 1;
        }
        assert_eq!(cnt, count);

        Ok(())
    }
}

mod gpkg {
    use futures_util::stream::TryStreamExt;
    use sqlx::prelude::*;
    use sqlx::sqlite::SqliteConnection;

    pub(super) async fn gpkg_to_geo(
        fpath: &str,
        table: &str,
        count: usize,
    ) -> std::result::Result<(), sqlx::Error> {
        use geozero_core::gpkg::geo::Geometry as GeoZeroGeometry;

        // ogr2ogr -f GPKG countries.gpkg countries.fgb

        let mut conn = SqliteConnection::connect(&format!("sqlite://{}", fpath)).await?;

        let sql = format!("SELECT geom FROM {}", table);
        let mut cursor = sqlx::query(&sql).fetch(&mut conn);
        let mut cnt = 0;
        while let Some(row) = cursor.try_next().await? {
            let _geom = row.get::<GeoZeroGeometry, _>(0);
            cnt += 1;
        }
        assert_eq!(cnt, count);
        Ok(())
    }

    pub(super) async fn gpkg_to_geo_bbox(
        fpath: &str,
        table: &str,
        min_x: f64,
        min_y: f64,
        max_x: f64,
        max_y: f64,
        count: usize,
    ) -> std::result::Result<(), sqlx::Error> {
        use geozero_core::gpkg::geo::Geometry as GeoZeroGeometry;

        let mut conn = SqliteConnection::connect(&format!("sqlite://{}", fpath)).await?;

        // http://erouault.blogspot.com/2017/03/dealing-with-huge-vector-geopackage.html
        let sql = format!(
            "SELECT geom FROM {} t JOIN rtree_{}_geom r ON t.fid = r.id
                         WHERE r.minx <= {} AND r.maxx >= {} AND
                               r.miny <= {} AND r.maxy >= {}",
            table, table, max_x, min_x, max_y, min_y
        );
        let mut cursor = sqlx::query(&sql).fetch(&mut conn);
        let mut cnt = 0;
        while let Some(row) = cursor.try_next().await? {
            let _geom = row.get::<GeoZeroGeometry, _>(0);
            cnt += 1;
        }
        assert_eq!(cnt, count);
        Ok(())
    }
}

fn countries_benchmark(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("countries");
    group.bench_function("postgis_sqlx", |b| {
        b.iter(|| rt.block_on(postgis_sqlx::postgis_sqlx_to_geo("countries", 179)));
    });
    group.bench_function("rust_postgis", |b| {
        b.iter(|| rust_postgis::rust_postgis_to_geo("countries", 179))
    });
    group.bench_function("postgis_postgres", |b| {
        b.iter(|| postgis_postgres::postgis_postgres_to_geo("countries", 179))
    });
    group.bench_function("fgb", |b| {
        b.iter(|| fgb::fgb_to_geo("tests/data/countries.fgb", 179))
    });
    group.bench_function("gpkg", |b| {
        b.iter(|| {
            rt.block_on(gpkg::gpkg_to_geo(
                "tests/data/countries.gpkg",
                "countries",
                179,
            ))
        })
    });
    group.finish()
}

fn countries_bbox_benchmark(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("countries_bbox");
    group.bench_function("postgis_postgres", |b| {
        b.iter(|| {
            postgis_postgres::postgis_postgres_to_geo_bbox(
                "countries",
                8.8,
                47.2,
                9.5,
                55.3,
                4326,
                6,
            )
        })
    });
    group.bench_function("gpkg", |b| {
        b.iter(|| {
            rt.block_on(gpkg::gpkg_to_geo_bbox(
                "tests/data/countries.gpkg",
                "countries",
                8.8,
                47.2,
                9.5,
                55.3,
                6,
            ))
        })
    });
    group.bench_function("fgb", |b| {
        b.iter(|| fgb::fgb_to_geo_bbox("tests/data/countries.fgb", 8.8, 47.2, 9.5, 55.3, 6))
    });
    group.finish()
}

fn buildings_benchmark(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("buildings");
    // 973.08 ms
    group.bench_function("fgb", |b| {
        b.iter(|| fgb::fgb_to_geo("tests/data/osm-buildings-3857-ch.fgb", 2407771))
    });
    // 6.0288 s
    group.bench_function("gpkg", |b| {
        b.iter(|| {
            rt.block_on(gpkg::gpkg_to_geo(
                "tests/data/osm-buildings-3857-ch.gpkg",
                "buildings",
                2407771,
            ))
        })
    });
    // 4.5416 s
    group.bench_function("postgis_postgres", |b| {
        b.iter(|| postgis_postgres::postgis_postgres_to_geo("buildings", 2407771))
    });
    group.bench_function("postgis_sqlx", |b| {
        b.iter(|| rt.block_on(postgis_sqlx::postgis_sqlx_to_geo("buildings", 2407771)))
    });
    // 4.4715 s
    group.bench_function("rust_postgis", |b| {
        b.iter(|| rust_postgis::rust_postgis_to_geo("buildings", 2407771))
    });
    group.finish()
}

fn buildings_bbox_benchmark(c: &mut Criterion) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    let mut group = c.benchmark_group("buildings_bbox");
    group.bench_function("gpkg", |b| {
        b.iter(|| {
            rt.block_on(gpkg::gpkg_to_geo_bbox(
                "tests/data/osm-buildings-3857-ch.gpkg",
                "buildings",
                939651.0,
                5997817.0,
                957733.0,
                6012256.0,
                54355, // fgb: 54351
            ))
        })
    });
    group.bench_function("fgb", |b| {
        b.iter(|| {
            fgb::fgb_to_geo_bbox(
                "tests/data/osm-buildings-3857-ch.fgb",
                939651.0,
                5997817.0,
                957733.0,
                6012256.0,
                54351,
            )
        })
    });

    group.bench_function("postgis_postgres", |b| {
        b.iter(|| {
            postgis_postgres::postgis_postgres_to_geo_bbox(
                "buildings",
                939651.0,
                5997817.0,
                957733.0,
                6012256.0,
                3857,
                54353, // fgb: 54351
            )
        })
    });
    group.finish()
}

criterion_group!(name=benches; config=Criterion::default().sample_size(10);
                 targets=countries_benchmark,countries_bbox_benchmark,buildings_bbox_benchmark,buildings_benchmark);
criterion_main!(benches);
