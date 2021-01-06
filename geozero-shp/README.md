# GeoZero Shapefile driver

Reading Shapefiles with [GeoZero](https://github.com/georust/geozero) API.

Features:
- [x] Read support for OGC simple feature types
- [x] Convert to GeoJSON, WKB (PostGIS/GeoPackage), WKT, GEOS, GDAL formats and more
- [ ] Support for Multipatch types
- [ ] Read spatial index
- [ ] Read projection files

For writing Shapefiles either use [shapefile-rs](https://crates.io/crates/shapefile) or the GDAL driver of [GeoZero Core](https://crates.io/crates/geozero-core)

Originally based on shapefile-rs from Thomas Montaigu.


## Usage example

```Rust
use geozero_core::geojson::GeoJsonWriter;

let reader = geozero_shp::Reader::from_path("poly.shp")?;
let mut json: Vec<u8> = Vec::new();
let cnt = reader.iter_features(GeoJsonWriter::new(&mut json))?.count();
```
