# GeoZero

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

[geozero-core](https://crates.io/crates/geozero-core):
* GeoJSON Reader + Writer
* [GEOS](https://github.com/georust/geos) Reader + Writer
* [GDAL](https://github.com/georust/gdal) geometry Reader + Writer
* WKB Reader supporting
  - PostGIS geometries for [rust-postgres](https://github.com/sfackler/rust-postgres) and [SQLx](https://github.com/launchbadge/sqlx)
  - GeoPackage geometries for [SQLx](https://github.com/launchbadge/sqlx)
* WKT Writer
* SVG Writer
* [geo-types](https://github.com/georust/geo) Writer

External:
* [FlatGeobuf Reader](https://github.com/bjornharrtell/flatgeobuf)

## Applications

* [flatgeobuf-gpu](https://github.com/pka/flatgeobuf-gpu): Demo rendering FlatGeobuf to GPU
* [flatgeobuf-wasm](https://github.com/pka/flatgeobuf-wasm): WASM demo displaying FlatGeobuf as SVG

## CLI

geozero includes an experimental command line interface for converting between supported formats.

## Usage examples

Count vertices of an input geometry:
```rust
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
Full source code: [geozero-api.rs](./geozero-core/tests/geozero-api.rs)

Find maximal height in 3D polygons:
```rust
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
while let Some(feature) = fgb.next()? {
    let geometry = feature.geometry().unwrap();
    geometry.process(&mut max_finder, GeometryType::MultiPolygon)?;
}
```
Full source code: [geozero-api.rs](./geozero-core/tests/geozero-api.rs)

Render polygons:
```rust
struct PathDrawer<'a> {
    canvas: &'a mut CanvasRenderingContext2D,
    path: Path2D,
}

impl<'a> GeomProcessor for PathDrawer<'a> {
    fn linestring_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        self.path = Path2D::new();
        Ok(())
    }
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
        self.canvas.fill_path(self.path.to_owned(), FillRule::Winding);
        Ok(())
    }
}
```
Full source code: [flatgeobuf-gpu](https://github.com/pka/flatgeobuf-gpu)

Read a FlatGeobuf dataset with async HTTP client applying a bbox filter and convert to GeoJSON:
```rust
let url = "https://github.com/pka/geozero/raw/master/geozero-core/tests/data/countries.fgb";
let mut fgb = HttpFgbReader::open(url).await?;
fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;

let mut fout = BufWriter::new(File::create("countries.json")?);
let mut json = GeoJsonWriter::new(&mut fout);
fgb.process_features(&mut json).await?;
```
Full source code: [geojson.rs](./geozero-core/tests/geojson.rs)

Read FlatGeobuf data as rustgeo geometries and calculate label position with [polylabel-rs](https://github.com/urschrei/polylabel-rs):
```rust
let mut file = BufReader::new(File::open("countries.fgb")?);
let mut fgb = FgbReader::open(&mut file)?;
fgb.select_all()?;
while let Some(feature) = fgb.next()? {
    let props = feature.properties()?;
    let geometry = feature.geometry().unwrap();
    let mut geo = Geo::new();
    geometry.process(&mut geo, GeometryType::MultiPolygon)?;
    if let Geometry::MultiPolygon(mpoly) = geo.geometry() {
        if let Some(poly) = &mpoly.0.iter().next() {
            let label_pos = polylabel(&poly, &0.10).unwrap();
            println!("{}: {:?}", props["name"], label_pos);
        }
    }
}
```
Full source code: [polylabel.rs](./geozero-core/tests/polylabel.rs)

Create a KD-tree index with [kdbush](https://github.com/pka/rust-kdbush):
```rust
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
read_geojson_geom(f, &mut points)?;
points.index.build_index();
```
Full source code: [kdbush.rs](./geozero-core/tests/kdbush.rs)


Use [GEOS](https://github.com/georust/geos) with prepared geometries:
```rust
let geojson = r#"{"type": "Polygon", "coordinates": [[[0, 0], [10, 0], [10, 6], [0, 6], [0, 0]]]}"#;
let mut geos = Geos::new();
read_geojson(geojson.as_bytes(), &mut geos).unwrap();
let prepared_geom = geos.geometry().to_prepared_geom().expect("to_prepared_geom failed");
let geom2 = geos::Geometry::new_from_wkt("POINT (2.5 2.5)").expect("Invalid geometry");
assert_eq!(prepared_geom.contains(&geom2), Ok(true));
```
Full source code: [geos.rs](./geozero-core/tests/geos.rs)
