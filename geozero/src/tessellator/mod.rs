use crate::error::Result;
use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use lyon::math::{point, Point};
use lyon::path::path::Builder;
use lyon::path::Path;
use lyon::tessellation::geometry_builder::simple_builder;
use lyon::tessellation::*;
use std::mem;

/// Triangle generator output
#[allow(unused_variables)]
pub trait VertexOutput {
    fn vertex(&self, x: f32, y: f32, z: f32) {}
    fn triangle(&self, idx0: u16, idx1: u16, idx2: u16) {}
}

/// Tessellator.
pub struct Tessellator<'a> {
    vertex_out: &'a dyn VertexOutput,
    has_started: bool,
    builder: Builder,
    num_rings: usize,
}

impl<'a> Tessellator<'a> {
    pub fn new(out: &'a dyn VertexOutput) -> Self {
        Tessellator {
            vertex_out: out,
            has_started: false,
            builder: Path::builder(),
            num_rings: 0,
        }
    }
}

impl GeomProcessor for Tessellator<'_> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        if idx == 0 {
            self.has_started = true;
            self.builder.begin(point(x as f32, y as f32));
        } else {
            self.builder.line_to(point(x as f32, y as f32));
        }
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }
    fn point_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }
    fn multipoint_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }
    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.num_rings = 1;
        }
        Ok(())
    }
    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        if self.has_started {
            self.has_started = false;
            self.builder.close();
        }
        if tagged {
            let builder = mem::replace(&mut self.builder, Path::builder());
            tessellate_line(&builder.build());
        }
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }
    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }
    fn polygon_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        self.num_rings = size;
        Ok(())
    }
    fn polygon_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        let mut builder = mem::replace(&mut self.builder, Path::builder());
        if self.has_started {
            self.has_started = false;
            builder.close();
        }
        tessellate_poly(&builder.build(), self.vertex_out);
        Ok(())
    }
}

fn tessellate_line(path: &Path) {
    let mut geometry: VertexBuffers<Point, u16> = VertexBuffers::new();
    let mut vertex_builder = simple_builder(&mut geometry);
    let mut tessellator = StrokeTessellator::new();
    tessellator
        .tessellate(path, &StrokeOptions::default(), &mut vertex_builder)
        .unwrap();
    println!(
        " -- {:?} vertices {:?} indices",
        geometry.vertices, geometry.indices
    );
}

fn tessellate_poly(path: &Path, out: &dyn VertexOutput) {
    let mut buffers: VertexBuffers<(), u16> = VertexBuffers::new();
    let mut tessellator = FillTessellator::new();
    tessellator
        .tessellate_path(
            path,
            &FillOptions::default(),
            &mut BuffersBuilder::new(&mut buffers, |pos: FillVertex| {
                let pos = pos.position();
                out.vertex(pos.x, pos.y, 0.0);
            }),
        )
        .unwrap();
    for tri in buffers.indices.chunks(3) {
        out.triangle(tri[0], tri[1], tri[2]);
    }
}

impl PropertyProcessor for Tessellator<'_> {}
impl FeatureProcessor for Tessellator<'_> {}

/// OBJ writer
pub struct ObjWriter;

impl VertexOutput for ObjWriter {
    fn vertex(&self, x: f32, y: f32, z: f32) {
        println!("v {x} {y} {z}");
    }
    fn triangle(&self, idx0: u16, idx1: u16, idx2: u16) {
        println!("f {} {} {}", idx0 + 1, idx1 + 1, idx2 + 1);
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::geojson::geojson_reader::read_geojson;

    #[test]
    fn point_geom() {
        let geojson = r#"{"type": "Point", "coordinates": [1, 1]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn multipoint_geom() {
        let geojson = r#"{"type": "MultiPoint", "coordinates": [[1, 1], [2, 2]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn multipoint_empty_geom() {
        let geojson = r#"{"type": "MultiPoint", "coordinates": []}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn line_geom() {
        let geojson = r#"{"type": "LineString", "coordinates": [[1,1], [2,2]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn line_empty_geom() {
        let geojson = r#"{"type": "LineString", "coordinates": []}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    // #[test]
    // fn line_geom_3d() {
    //     let geojson = r#"{"type": "LineString", "coordinates": [[1,1,10], [2,2,20]]}"#;
    //     let out = ObjWriter {};
    //     let mut tessellator = Tessellator::new(&out);
    //     assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    // }

    #[test]
    fn multiline_geom() {
        let geojson =
            r#"{"type": "MultiLineString", "coordinates": [[[1,1],[2,2]],[[3,3],[4,4]]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn multiline_empty_geom() {
        let geojson = r#"{"type": "MultiLineString", "coordinates": [[],[]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());

        let geojson = r#"{"type": "MultiLineString", "coordinates": []}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn polygon_geom() {
        let geojson = r#"{"type": "Polygon", "coordinates": [[[0, 0], [0, 3], [3, 3], [3, 0], [0, 0]],[[0.2, 0.2], [0.2, 2], [2, 2], [2, 0.2], [0.2, 0.2]]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn polygon_empty_geom() {
        let geojson = r#"{"type": "Polygon", "coordinates": [[],[]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());

        let geojson = r#"{"type": "Polygon", "coordinates": []}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn multipolygon_geom() {
        let geojson =
            r#"{"type": "MultiPolygon", "coordinates": [[[[0,0],[0,1],[1,1],[1,0],[0,0]]]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    #[test]
    fn multipolygon_empty_geom() {
        let geojson = r#"{"type": "MultiPolygon", "coordinates": [[[]]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
        let geojson = r#"{"type": "MultiPolygon", "coordinates": [[]]}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
        let geojson = r#"{"type": "MultiPolygon", "coordinates": []}"#;
        let out = ObjWriter {};
        let mut tessellator = Tessellator::new(&out);
        assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    }

    // #[test]
    // fn geometry_collection_geom() {
    //     let geojson = r#"{"type": "Point", "coordinates": [1, 1]}"#;
    //     let out = ObjWriter {};
    //     let mut tessellator = Tessellator::new(&out);
    //     assert!(read_geojson(geojson.as_bytes(), &mut tessellator).is_ok());
    // }
}
