# GeoZero benchmark

## Benchmarks

* `countries`: Read all countries of the world as generalized polygons (GPKG size: 324KB)
* `countries_bbox`: Read 6/179 countries within a bounding box
* `buildings`: Read 2.4 Mio OSM building polygons (GPKG size: 764MB)
* `buildings_bbox`: Read 55'000 OSM building polygons within a bounding box

## Tested configurations


*  `shp`: Shapefile with GDAL driver
*  `shp_geom`: Shapefile with GDAL driver (geometries only)
*  `fgb`: FlatGeobuf file with Rust driver and GeoZero processor for `geo`
*  `gpkg`: GeoPackage file with GeoZero GPKG reader for [SQLx](https://github.com/launchbadge/sqlx) and processor for `geo`
*  `gpkg_gdal`: GeoPackage file with GDAL driver
*  `geojson`: GeoJSON file with GDAL driver
*  `fgb_http`: FlatGeobuf over HTTP (Nginx) with Rust driver and GeoZero processor for `geo`
*  `postgis_sqlx`: PostgreSQL DB with GeoZero PostGIS reader for [SQLx](https://github.com/launchbadge/sqlx) and processor for `geo`
*  `postgis_postgres`: PostgreSQL DB with GeoZero PostGIS reader for [rust-postgres](https://github.com/sfackler/rust-postgres) and processor for `geo`
*  `rust_postgis`: PostgreSQL DB with [rust-postgis](https://github.com/andelf/rust-postgis) reader for [rust-postgres](https://github.com/sfackler/rust-postgres)

Remark: All data is converted from the FlatGeobuf file and therefore has the same ordering.

## Results

![countries](./results/200617/countries/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| countries/1-shp | 1.33 |
| countries/2-shp_geom | 1.24 |
| countries/3-fgb | 0.19 |
| countries/4-gpkg | 1.12 |
| countries/5-geojson | 26.58 |
| countries/6-fgb_http | 9.84 |
| countries/7-postgis_sqlx | 24.09 |
| countries/8-postgis_postgres | 11.85 |
| countries/9-rust_postgis | 12.05 |

![countries_bbox](./results/200617/countries_bbox/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| countries_bbox/1-shp | 1.15 |
| countries_bbox/2-shp_geom | 1.16 |
| countries_bbox/3-fgb | 0.03 |
| countries_bbox/4-gpkg | 0.66 |
| countries_bbox/5-gpkg_gdal | 2.81 |
| countries_bbox/6-fgb_http | 9.91 |
| countries_bbox/7-postgis_sqlx | 23.52 |
| countries_bbox/8-postgis_postgres | 11.28 |
| countries_bbox/9-rust_postgis | 11.30 |

![buildings](./results/200617/buildings/violin.svg)

| Benchmark | Median (s) |
| --------- | ---------: |
| buildings/1-shp | 6.08 |
| buildings/2-shp_geom | 1.78 |
| buildings/3-fgb | 0.96 |
| buildings/4-gpkg | 4.81 |
| buildings/5-fgb_http | 2.26 |
| buildings/7-postgis_postgres | 3.21 |

![buildings_bbox](./results/200617/buildings_bbox/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| buildings_bbox/1-shp | 169.53 |
| buildings_bbox/2-shp_geom | 64.85 |
| buildings_bbox/3-fgb | 69.53 |
| buildings_bbox/4-gpkg | 107.84 |
| buildings_bbox/5-gpkg_gdal | 141.35 |
| buildings_bbox/6-fgb_http | 58.80 |
| buildings_bbox/7-postgis_sqlx | 147.29 |
| buildings_bbox/8-postgis_postgres | 119.08 |
| buildings_bbox/9-rust_postgis | 126.88 |

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
