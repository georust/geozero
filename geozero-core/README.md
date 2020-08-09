# GeoZero Core

Collection of [GeoZero](https://github.com/pka/geozero) processors for zero-copy reading and writing of geospatial data.

[Changelog](https://github.com/pka/geozero/blob/master/geozero-core/CHANGELOG.md)


## GeoZero API implementations

* GeoJSON Reader + Writer
* [GEOS](https://github.com/georust/geos) Reader + Writer
* [GDAL](https://github.com/georust/gdal) geometry Reader + Writer
* WKB Reader supporting
  - PostGIS geometries for [rust-postgres](https://github.com/sfackler/rust-postgres) and [SQLx](https://github.com/launchbadge/sqlx)
  - GeoPackage geometries for [SQLx](https://github.com/launchbadge/sqlx)
* WKT Writer
* SVG Writer
* [geo-types](https://github.com/georust/geo) Writer


## Build Features

* `gdal-lib`: Include [GEOS](https://github.com/georust/geos) processing
* `geos-lib`: Include [GDAL](https://github.com/georust/gdal) processing
* `gpkg`: Include GeoPackage implementation for [SQLx](https://github.com/launchbadge/sqlx)
* `postgis-sqlx`: Include PostGIS implementation for [SQLx](https://github.com/launchbadge/sqlx)
* `postgis-postgres`: Include PostGIS implementation for [rust-postgres](https://github.com/sfackler/rust-postgres)
