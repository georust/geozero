## 0.9.0 (UNRELEASED)

* Add MVT writer and reader
* Return ColumnNotFound instead of ColumnUnknown for Null values
* Update to gdal 0.12 / gdal-sys 0.6

## 0.8.0 (2021-12-02)

* Breaking: Property reader returns error instead of None
* Update to gdal 0.11 / gdal-sys 0.5
* Update to geos 8.0

## 0.7.7 (2021-11-09)

* Support XYZ in GeoJSON writer

## 0.7.6 (2021-11-09)

* Support XYZ in GeoJSON reader

## 0.7.5 (2021-11-04)

* Process properties in GeoJSON reader

## 0.7.4 (2021-03-10)

* Support for SQLx macros
* Provide macros for PostGIS/GPKG-Encoding/Decoding

## 0.7.3 (2021-02-27)

* Fix docs.rs build

## 0.7.1 (2021-02-27)

* Split wkb::Geometry into wkb::Encode and wkb::Decode
  with support for NULL value decoding
* Implement FromWkb for GeoJsonString, GDAL, SvgString and WktString
* Improve crate documentation

## 0.7.0 (2021-02-24)

* Merge geozero-core into geozero crate
* Format conversion traits
* Add feature flags for all formats
* Add geo-types reader
* Implement geo-types to Postgis/GPKG WKB encoding
* Update sqlx, postgres and bytes dependencies

## 0.6.1 (2021-01-06)

* Fix docs.rs

## 0.6.0 (2021-01-05)

* Feature access API
* Add multi_dim function to geometry API
* Update to georust 0.6.0

## 0.5.2 (2020-12-24)

* Update URLs to GeoRust organisation
* Update to georust/gdal 0.7
* Update to geojson 0.21
* Add experimental tesselator

## 0.5.1 (2020-07-09)

* Support untagged triangles as parts of a TIN
* Activate all features for doc.rs

## 0.5.0 (2020-07-03)

* Add curve type processing methods
* Add triangle/polyhedralsurface/tin processing methods
* Add support for seek_bufread::BufReader
* Add WKB writer with PostGIS and GeoPackage support
* Add PostGIS + GeoPackage encoder and decoder for GEOS geometry
* Support curve and surface types in WKT writer and WKB reader/writer
* Update to SQLx 0.4-beta
* Update to GEOS bindings 7.0
* Rename geo::RustGeo to geo_types::Geo

## 0.4.3 (2020-06-05)

* Add GeoPackage WKB reader
* Add GeoPackage decoder for geo-types geometry (SQLx)

## 0.4.2 (2020-06-04)

* Support GeometryCollection in GeoJSON reader

## 0.4.1 (2020-06-04)

* Support for GeometryCollection

## 0.4.0 (2020-06-03)

* Add GeometryCollection processing methods
* Add SRID processing method
* Add WKB reader
* Add PostGIS decoder for geo-types geometry (rust-postgres and SQLx)
* Add GDAL geometry reader + writer

## 0.3.1 (2020-05-25)

* Add GEOS reader + writer

## 0.3.0 (2020-05-09)

* Add geo-types writer
* Impl FeatureProcessor for GeoJSON reader
* Change Reader::open to Read + Seek trait 

## 0.2.0 (2020-04-20)

First publication on crates.io
