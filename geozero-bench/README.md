# GeoZero benchmark

## Benchmarks

* `countries`: Read all countries of the world as generalized polygons (GPKG size: 324KB)
* `countries_bbox`: Read 6/179 countries within a bounding box
* `buildings`: Read 2.4 Mio OSM building polygons (GPKG size: 764MB)
* `buildings_bbox`: Read 55'000 OSM building polygons within a bounding box

## Tested configurations

*  `fgb`: FlatGeobuf file with Rust driver and GeoZero processor for `geo`
*  `gpkg`: GeoPackage file with GeoZero GPKG reader for [SQLx](https://github.com/launchbadge/sqlx) and processor for `geo`
*  `fgb_http`: FlatGeobuf over HTTP (Nginx) with Rust driver and GeoZero processor for `geo`
*  `postgis_sqlx`: PostgreSQL DB with GeoZero PostGIS reader for [SQLx](https://github.com/launchbadge/sqlx) and processor for `geo`
*  `postgis_postgres`: PostgreSQL DB with GeoZero PostGIS reader for [rust-postgres](https://github.com/sfackler/rust-postgres) and processor for `geo`
*  `rust_postgis`: PostgreSQL DB with [rust-postgis](https://github.com/andelf/rust-postgis) reader for [rust-postgres](https://github.com/sfackler/rust-postgres)

Remark: All data is converted from the FlatGeobuf file and therefore has the same ordering.

## Results

![countries](./results/200615/countries/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| countries/fgb | 0.19 |
| countries/fgb_http | 10.65 |
| countries/gpkg | 1.13 |
| countries/postgis_postgres | 12.35 |
| countries/postgis_sqlx | 25.32 |
| countries/rust_postgis | 12.54 |

![countries_bbox](./results/200615/countries_bbox/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| countries_bbox/fgb | 0.03 |
| countries_bbox/fgb_http | 10.61 |
| countries_bbox/gpkg | 0.68 |
| countries_bbox/postgis_postgres | 11.90 |

![buildings](./results/200615/buildings/violin.svg)

| Benchmark | Median (s) |
| --------- | ---------: |
| buildings/fgb | 0.98 |
| buildings/fgb_http | 2.36 |
| buildings/gpkg | 4.52 |
| buildings/postgis_postgres | 3.29 |
| buildings/postgis_sqlx | 4.60 |
| buildings/rust_postgis | 3.51 |

![buildings_bbox](./results/200615/buildings_bbox/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| buildings_bbox/fgb | 71.27 |
| buildings_bbox/fgb_http | 63.89 |
| buildings_bbox/gpkg | 112.26 |
| buildings_bbox/postgis_postgres | 125.73 |

## Running the benchmark

Prepare data:

    cd tests/data
    make

Create PostGIS database:

    make createdb
    make countries_table osm_buildings_table

Start web server:

    docker-compose up -d
    cd ../..

Run benchmark:

    export DATABASE_URL=postgresql://$USER@localhost/geozerobench
    cargo bench
