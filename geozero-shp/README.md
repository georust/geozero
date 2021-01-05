# GeoZero Shapefile driver

Reading Shapefiles with [GeoZero](https://github.com/georust/geozero) API.

Planned functions:
* Support for Multipatch types
* Read spatial index

For writing Shapefiles either use [shapefile-rs](https://crates.io/crates/shapefile) or the GDAL driver of [GeoZero Core](https://crates.io/crates/geozero-core)

Originally based on shapefile-rs from Thomas Montaigu.


## Usage example

```Rust
use geozero_core::geojson::GeoJsonWriter;

let reader = geozero_shp::Reader::from_path("poly.shp")?;
let mut json: Vec<u8> = Vec::new();
let cnt = reader.iter_features(GeoJsonWriter::new(&mut json))?.count();
```
