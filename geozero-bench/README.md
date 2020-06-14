# GeoZero benchmark

## Results

![countries](./results/200614/countries/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| countries/fgb | 0.20 |
| countries/gpkg | 1.47 |
| countries/postgis_postgres | 12.02 |
| countries/postgis_sqlx | 18.81 |
| countries/rust_postgis | 10.55 |

![countries_bbox](./results/200614/countries_bbox/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| countries_bbox/fgb | 0.03 |
| countries_bbox/gpkg | 1.02 |
| countries_bbox/postgis_postgres | 11.22 |

![buildings](./results/200614/buildings/violin.svg)

| Benchmark | Median (s) |
| --------- | ---------: |
| buildings/fgb | 0.94 |
| buildings/gpkg | 4.84 |
| buildings/postgis_postgres | 3.23 |
| buildings/postgis_sqlx | 4.52 |
| buildings/rust_postgis | 3.29 |

![buildings_bbox](./results/200614/buildings_bbox/violin.svg)

| Benchmark | Median (ms) |
| --------- | ----------: |
| buildings_bbox/fgb | 68.50 |
| buildings_bbox/gpkg | 109.95 |
| buildings_bbox/postgis_postgres | 132.20 |
