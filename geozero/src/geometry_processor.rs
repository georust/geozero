use crate::error::Result;

/// Dimensions requested for processing
#[derive(Default, Clone, Copy)]
pub struct CoordDimensions {
    /// height
    pub z: bool,
    /// measurement
    pub m: bool,
    /// geodetic decimal year time
    pub t: bool,
    /// time nanosecond measurement
    pub tm: bool,
}

/// Geometry processing trait
#[allow(unused_variables)]
pub trait GeomProcessor {
    /// Additional dimensions requested when processing coordinates
    fn dimensions(&self) -> CoordDimensions {
        CoordDimensions {
            z: false,
            m: false,
            t: false,
            tm: false,
        }
    }

    /// SRID of geometries
    ///
    /// Emitted before geometry begin
    fn srid(&mut self, srid: Option<i32>) -> Result<()> {
        Ok(())
    }

    /// Process coordinate with x,y dimensions
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Process coordinate with all requested dimensions
    fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) -> Result<()> {
        Ok(())
    }

    /// Begin of Point processing
    ///
    /// Next: xy/coordinate
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of Point processing
    fn point_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of MultiPoint processing
    ///
    /// Next: size * xy/coordinate
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiPoint processing
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of LineString processing
    ///
    /// An untagged LineString is either a Polygon ring or part of a MultiLineString
    ///
    /// Next: size * xy/coordinate
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of LineString processing
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of MultiLineString processing
    ///
    /// Next: size * LineString (untagged)
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiLineString processing
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of Polygon processing
    ///
    /// An untagged Polygon is part of a MultiPolygon
    ///
    /// Next: size * LineString (untagged) = rings
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of Polygon processing
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of MultiPolygon processing
    ///
    /// Next: size * Polygon (untagged)
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiPolygon processing
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of GeometryCollection processing
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of GeometryCollection processing
    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of CircularString processing
    ///
    /// The CircularString is the basic curve type, similar to a LineString in the linear world. A single segment required three points, the start and end points (first and third) and any other point on the arc. The exception to this is for a closed circle, where the start and end points are the same. In this case the second point MUST be the center of the arc, ie the opposite side of the circle. To chain arcs together, the last point of the previous arc becomes the first point of the next arc, just like in LineString. This means that a valid circular string must have an odd number of points greated than 1.
    ///
    /// Next: size * xy/coordinate
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of CircularString processing
    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of CompoundCurve processing
    ///
    /// A compound curve is a single, continuous curve that has both curved (circular) segments and linear segments. That means that in addition to having well-formed components, the end point of every component (except the last) must be coincident with the start point of the following component.
    ///
    /// Next: size * (CircularString | LineString (untagged))
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of CompoundCurve processing
    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of CurvePolygon processing
    ///
    /// A CurvePolygon is just like a polygon, with an outer ring and zero or more inner rings. The difference is that a ring can take the form of a circular string, linear string or compound string.
    ///
    /// Next: size * (CircularString | LineString (untagged) | CompoundCurve)
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of CurvePolygon processing
    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of MultiCurve processing
    ///
    /// The MultiCurve is a collection of curves, which can include linear strings, circular strings or compound strings.
    ///
    /// Next: size * (CircularString | LineString (untagged) | CompoundCurve)
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiCurve processing
    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of MultiSurface processing
    ///
    /// The MultiSurface is a collection of surfaces, which can be (linear) polygons or curve polygons.
    ///
    /// Next: size * (CurvePolygon | Polygon (untagged))
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiSurface processing
    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }
    /// Begin of Triangle processing
    ///
    /// Next: size * LineString (untagged) = rings
    fn triangle_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of Polygon processing
    fn triangle_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of PolyhedralSurface processing
    ///
    /// Next: size * Polygon (untagged)
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiPolygon processing
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of Tin processing
    ///
    /// Next: size * Polygon (untagged)
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiPolygon processing
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }
}

#[test]
fn error_message() {
    use crate::error::GeozeroError;
    struct Test;
    impl GeomProcessor for Test {
        fn linestring_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> Result<()> {
            Err(GeozeroError::Geometry("test".to_string()))
        }
    }
    assert_eq!(
        Test {}
            .linestring_begin(false, 0, 0)
            .err()
            .unwrap()
            .to_string(),
        "processing geometry `test`".to_string()
    );
}
