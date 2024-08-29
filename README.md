# GeoZero

[![GitHub](https://img.shields.io/badge/github-georust/geozero-8da0cb?logo=github)](https://github.com/georust/geozero)
[![CI build](https://github.com/georust/geozero/workflows/CI-Linux/badge.svg)](https://github.com/georust/geozero/actions)
[![crates.io version](https://img.shields.io/crates/v/geozero.svg)](https://crates.io/crates/geozero)
[![docs.rs docs](https://docs.rs/geozero/badge.svg)](https://docs.rs/geozero)
[![Discord Chat](https://img.shields.io/discord/598002550221963289.svg)](https://discord.gg/Fp2aape)

Zero-Copy reading and writing of geospatial data.

GeoZero defines an API for reading geospatial data formats without an intermediate representation.
It defines traits which can be implemented to read and convert to an arbitrary format
or render geometries directly.

Supported geometry types:
* [OGC Simple Features](https://en.wikipedia.org/wiki/Simple_Features)
* Circular arcs as defined by SQL-MM Part 3
* TIN

Supported dimensions: X, Y, Z, M, T

## Available implementations

| Format  | Read | Write | Notes |
|---------|------|-------|-------|
| GeoJSON | ✅ | ✅ | |
| [GEOS](https://github.com/georust/geos) | ✅ | ✅ | |
| [GDAL](https://github.com/georust/gdal) | ✅ | ✅ | |
| WKB | ✅ | ✅ | Supports PostGIS geometries for [rust-postgres](https://github.com/sfackler/rust-postgres), [SQLx](https://github.com/launchbadge/sqlx) and [Diesel](https://github.com/diesel-rs/diesel). And also supports GeoPackage geometries for [SQLx](https://github.com/launchbadge/sqlx). |
| [WKT](https://github.com/georust/wkt) | ✅ | ✅ | |
| CSV | ✅ | ✅ | |
| SVG | ❌ | ✅ | |
| [geo-types](https://github.com/georust/geo) | ✅ | ✅ | |
| MVT (Mapbox Vector Tiles) | ✅ | ✅ | |
| GPX | ✅ | ❌ | |
| Shapefile | ✅ | ❌ | Available via the [geozero-shp](https://crates.io/crates/geozero-shp) crate. |
| FlatGeobuf | ✅ | ❌ | Available via the [flatgeobuf](https://crates.io/crates/flatgeobuf) crate. |
| GeoArrow | ✅ | ✅ | Available via the [geoarrow](https://crates.io/crates/geoarrow) crate. |
| GeoParquet | ✅ | ✅ | Available via the [geoarrow](https://crates.io/crates/geoarrow) crate. |

## Conversion API

Convert a GeoJSON polygon to geo-types and calculate centroid:
```rust,ignore
let geojson = GeoJson(r#"{"type": "Polygon", "coordinates": [[[0, 0], [10, 0], [10, 6], [0, 6], [0, 0]]]}"#);
if let Ok(Geometry::Polygon(poly)) = geojson.to_geo() {
    assert_eq!(poly.centroid().unwrap(), Point::new(5.0, 3.0));
}
```
Full source code: [geo_types.rs](./geozero/tests/geo_types.rs)


Convert GeoJSON to a [GEOS](https://github.com/georust/geos) prepared geometry:
```rust,ignore
let geojson = GeoJson(r#"{"type": "Polygon", "coordinates": [[[0, 0], [10, 0], [10, 6], [0, 6], [0, 0]]]}"#);
let geom = geojson.to_geos().expect("GEOS conversion failed");
let prepared_geom = geom.to_prepared_geom().expect("to_prepared_geom failed");
let geom2 = geos::Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
assert_eq!(prepared_geom.contains(&geom2), Ok(true));
```
Full source code: [geos.rs](./geozero/tests/geos.rs)


Read FlatGeobuf subset as GeoJSON:
```rust,ignore
let mut file = BufReader::new(File::open("countries.fgb")?);
let mut fgb = FgbReader::open(&mut file)?.select_bbox(8.8, 47.2, 9.5, 55.3)?;
println!("{}", fgb.to_json()?);
```
Full source code: [geojson.rs](./geozero/tests/geojson.rs)


## PostGIS usage examples

Select and insert geo-types geometries with rust-postgres. Requires the `with-postgis-postgres` feature:
```rust,ignore
let mut client = Client::connect(&std::env::var("DATABASE_URL").unwrap(), NoTls)?;

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
}

// Insert geometry
let geom: geo_types::Geometry<f64> = geo::Point::new(1.0, 3.0).into();
let _ = client.execute(
    "INSERT INTO point2d (datetimefield,geom) VALUES(now(),ST_SetSRID($1,4326))",
    &[&wkb::Encode(geom)],
);
```

Select and insert geo-types geometries with SQLx. Requires the `with-postgis-sqlx` feature:
```rust,ignore
let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(&env::var("DATABASE_URL").unwrap())
    .await?;

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
}

// Insert geometry
let geom: geo_types::Geometry<f64> = geo::Point::new(10.0, 20.0).into();
let _ = sqlx::query(
    "INSERT INTO point2d (datetimefield,geom) VALUES(now(),ST_SetSRID($1,4326))",
)
.bind(wkb::Encode(geom))
.execute(&pool)
.await?;
```

Using compile-time verification requires [type overrides](https://docs.rs/sqlx/latest/sqlx/macro.query.html#force-a-differentcustom-type):
```rust,ignore
let _ = sqlx::query!(
    "INSERT INTO point2d (datetimefield, geom) VALUES(now(), $1::geometry)",
    wkb::Encode(geom) as _
)
.execute(&pool)
.await?;

struct PointRec {
    pub geom: wkb::Decode<geo_types::Geometry<f64>>,
    pub datetimefield: Option<OffsetDateTime>,
}
let rec = sqlx::query_as!(
    PointRec,
    r#"SELECT datetimefield, geom as "geom!: _" FROM point2d"#
)
.fetch_one(&pool)
.await?;
assert_eq!(
    rec.geom.geometry.unwrap(),
    geo::Point::new(10.0, 20.0).into()
);
```

Full source code: [postgis.rs](./geozero/tests/postgis.rs)


## Processing API

Count vertices of an input geometry:
```rust,ignore
struct VertexCounter(u64);

impl GeomProcessor for VertexCounter {
    fn xy(&mut self, _x: f64, _y: f64, _idx: usize) -> Result<()> {
        self.0 += 1;
        Ok(())
    }
}

let mut vertex_counter = VertexCounter(0);
geometry.process(&mut vertex_counter, GeometryType::MultiPolygon)?;
```
Full source code: [geozero-api.rs](./geozero/tests/geozero-api.rs)

Find maximal height in 3D points:
```rust,ignore
struct MaxHeightFinder(f64);

impl GeomProcessor for MaxHeightFinder {
    fn coordinate(&mut self, _x: f64, _y: f64, z: Option<f64>, _m: Option<f64>, _t: Option<f64>, _tm: Option<u64>, _idx: usize) -> Result<()> {
        if let Some(z) = z {
            if z > self.0 {
                self.0 = z
            }
        }
        Ok(())
    }
}

let mut max_finder = MaxHeightFinder(0.0);
points.process_geom(&mut max_finder)?;
assert_eq!(max_finder.0, 457.1);
```
Full source code: [geozero-api.rs](./geozero/tests/geozero-api.rs)

Render polygons:
```rust,ignore
struct PathDrawer<'a> {
    canvas: &'a mut CanvasRenderingContext2D,
    path: Path2D,
}

impl<'a> GeomProcessor for PathDrawer<'a> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        if idx == 0 {
            self.path.move_to(vec2f(x, y));
        } else {
            self.path.line_to(vec2f(x, y));
        }
        Ok(())
    }
    fn linestring_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        self.path.close_path();
        self.canvas.fill_path(
            mem::replace(&mut self.path, Path2D::new()),
            FillRule::Winding,
        );
        Ok(())
    }
}
```
Full source code: [flatgeobuf-gpu](https://github.com/pka/flatgeobuf-gpu)

Read a FlatGeobuf dataset with async HTTP client applying a bbox filter and convert to GeoJSON:
```rust,ignore
let url = "https://flatgeobuf.org/test/data/countries.fgb";
let mut fgb = HttpFgbReader::open(url)
    .await?
    .select_bbox(8.8, 47.2, 9.5, 55.3)
    .await?;

let mut fout = BufWriter::new(File::create("countries.json")?);
let mut json = GeoJsonWriter::new(&mut fout);
fgb.process_features(&mut json).await?;
```
Full source code: [geojson.rs](./geozero/tests/geojson.rs)


Create a KD-tree index with [kdbush](https://github.com/pka/rust-kdbush):
```rust,ignore
struct PointIndex {
    pos: usize,
    index: KDBush,
}

impl geozero::GeomProcessor for PointIndex {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        self.index.add_point(self.pos, x, y);
        self.pos += 1;
        Ok(())
    }
}

let mut points = PointIndex {
    pos: 0,
    index: KDBush::new(1249, DEFAULT_NODE_SIZE),
};
read_geojson_geom(&mut f, &mut points)?;
points.index.build_index();
```
Full source code: [kdbush.rs](./geozero/tests/kdbush.rs)
