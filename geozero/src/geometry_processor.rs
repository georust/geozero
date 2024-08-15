use crate::error::{GeozeroError, Result};
use crate::WrappedXYProcessor;

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

impl CoordDimensions {
    pub const fn xy() -> Self {
        CoordDimensions {
            z: false,
            m: false,
            t: false,
            tm: false,
        }
    }
    pub const fn xyz() -> Self {
        CoordDimensions {
            z: true,
            m: false,
            t: false,
            tm: false,
        }
    }
    pub const fn xyzm() -> Self {
        CoordDimensions {
            z: true,
            m: true,
            t: false,
            tm: false,
        }
    }
    pub const fn xym() -> Self {
        CoordDimensions {
            z: false,
            m: true,
            t: false,
            tm: false,
        }
    }
}

/// Geometry processing trait
///
/// # Usage example:
///
/// ```rust
/// use geozero::{GeomProcessor, error::Result};
///
/// struct CoordPrinter;
///
/// impl GeomProcessor for CoordPrinter {
///     fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
///         Ok(println!("({x} {y})"))
///     }
/// }
/// ```
#[allow(unused_variables)]
pub trait GeomProcessor {
    /// Additional dimensions requested when processing coordinates
    fn dimensions(&self) -> CoordDimensions {
        CoordDimensions::xy()
    }

    /// Request additional dimensions for coordinate processing
    fn multi_dim(&self) -> bool {
        let dimensions = self.dimensions();
        dimensions.z || dimensions.m || dimensions.t || dimensions.tm
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

    /// Process empty coordinates, like WKT's `POINT EMPTY`
    ///
    /// - `idx` is the positional index inside this geometry. `idx` will usually be 0 except in the
    ///   case of a MultiPoint or GeometryCollection.
    fn empty_point(&mut self, idx: usize) -> Result<()> {
        Err(GeozeroError::Geometry(
            "The input was an empty Point, but the output doesn't support empty Points".to_string(),
        ))
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
    /// Next: `size` calls to [`xy()`][`Self::xy()`] or [`coordinate()`][`Self::coordinate()`]
    ///
    /// ## Parameters
    ///
    /// - `size`: the number of Points in this MultiPoint
    /// - `idx`: the positional index of this MultiPoint. This will be 0 except in the case of a
    ///   GeometryCollection.
    ///
    /// ## Following events
    ///
    /// - `size` calls to [`xy()`][`Self::xy()`] or [`coordinate()`][`Self::coordinate()`] for each point.
    /// - [`multipoint_end`][Self::multipoint_end()] to end this MultiPoint
    ///
    /// As of v0.12, `point_begin` and `point_end` are **not** called for each point in a
    /// MultiPoint. See also discussion in [#184](https://github.com/georust/geozero/issues/184).
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of MultiPoint processing
    ///
    /// - `idx`: the positional index of this MultiPoint. This will be 0 except in the case of a
    ///   GeometryCollection.
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of `LineString` processing
    ///
    /// ## Parameters
    ///
    /// - `tagged`: if `false`, this `LineString` is either a Polygon ring or part of a `MultiLineString`
    /// - `size`: the number of coordinates in this LineString
    /// - `idx`: the positional index of this LineString. This will be 0 for a tagged LineString
    ///   except in the case of a GeometryCollection. This can be non-zero for an untagged
    ///   LineString for MultiLineStrings or Polygons with multiple interiors.
    ///
    /// ## Following events
    ///
    /// - `size` calls to [`xy()`][`Self::xy()`] or [`coordinate()`][`Self::coordinate()`] for each coordinate.
    /// - [`linestring_end`][Self::linestring_end()] to end this LineString
    fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of `LineString` processing
    fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of `MultiLineString` processing
    ///
    /// Next: size * LineString (untagged)
    ///
    /// ## Following events
    ///
    /// - `size` calls to:
    ///     - [`linestring_begin`][Self::linestring_begin] (with `tagged` set to `false`).
    ///     - one or more calls to [`xy()`][`Self::xy()`] or [`coordinate()`][`Self::coordinate()`] for each coordinate in the LineString.
    ///     - [`linestring_end`][Self::linestring_end]
    /// - [`multilinestring_end`][Self::multilinestring_end()] to end this MultiLineString
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of `MultiLineString` processing
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of `Polygon` processing
    ///
    /// ## Parameters
    ///
    /// - `tagged`: if `false`, this `Polygon` is part of a `MultiPolygon`.
    /// - `size`: the number of rings in this Polygon, _including_ the exterior ring.
    /// - `idx`: the positional index of this Polygon. This will be 0 for a tagged Polygon
    ///   except in the case of a GeometryCollection. This can be non-zero for an untagged
    ///   Polygon for a MultiPolygon with multiple interiors
    ///
    /// ## Following events
    ///
    /// - `size` calls to:
    ///     - [`linestring_begin`][Self::linestring_begin] (with `tagged` set to `false`).
    ///     - one or more calls to [`xy()`][`Self::xy()`] or [`coordinate()`][`Self::coordinate()`] for each coordinate in the ring.
    ///     - [`linestring_end`][Self::linestring_end]
    /// - [`polygon_end`][Self::polygon_end()] to end this Polygon
    fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of Polygon processing
    fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of `MultiPolygon` processing
    ///
    /// ## Parameters
    ///
    /// - `size`: the number of Polygons in this MultiPolygon.
    /// - `idx`: the positional index of this MultiPolygon. This will be 0 except in the case of a
    ///   GeometryCollection.
    ///
    /// ## Following events
    ///
    /// - `size` calls to:
    ///     - [`polygon_begin`][Self::polygon_begin] (with `tagged` set to `false`).
    ///     - See [`polygon_begin`][Self::polygon_begin] for its internal calls.
    ///     - [`polygon_end`][Self::polygon_end]
    /// - [`multipolygon_end`][Self::multipolygon_end()] to end this MultiPolygon
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of `MultiPolygon` processing
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of `GeometryCollection` processing
    ///
    /// ## Parameters
    ///
    /// - `size`: the number of geometries in this GeometryCollection.
    /// - `idx`: the positional index of this GeometryCollection. This can be greater than 0 for
    ///   nested geometry collections but also when using `GeometryProcessor` to process a
    ///   `Feature` whose geometry is a `GeometryCollection`. For an example of this see [this
    ///   comment](https://github.com/georust/geozero/pull/183#discussion_r1454319662).
    ///
    /// ## Following events
    ///
    /// - `size` calls to one of the internal geometry `begin` and `end` methods, called in pairs.
    /// - [`geometrycollection_end`][Self::geometrycollection_end()] to end this GeometryCollection
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of `GeometryCollection` processing
    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of `CircularString` processing
    ///
    /// The `CircularString` is the basic curve type, similar to a `LineString` in the linear world.
    /// A single segment required three points, the start and end points (first and third) and any other point on the arc.
    /// The exception to this is for a closed circle, where the start and end points are the same.
    /// In this case the second point MUST be the center of the arc, ie the opposite side of the circle.
    /// To chain arcs together, the last point of the previous arc becomes the first point of the next arc,
    /// just like in LineString. This means that a valid circular string must have an odd number of points greater than 1.
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
    /// An untagged Triangle is part of a Tin
    ///
    /// Next: size * LineString (untagged) = rings
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of Triangle processing
    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of PolyhedralSurface processing
    ///
    /// Next: size * Polygon (untagged)
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of PolyhedralSurface processing
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Begin of Tin processing
    ///
    /// Next: size * Polygon (untagged)
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        Ok(())
    }

    /// End of Tin processing
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        Ok(())
    }

    /// Combinator which inserts a call to `transform_xy` during processing, before [GeomProcessor::xy]
    /// or [GeomProcessor::coordinate] is called.
    ///
    /// Useful for pipelining multiple processors, e.g. to project your coordinates before outputting
    /// to a particular format.
    ///
    /// ```
    /// use geozero::geojson::GeoJson;
    /// use geozero::wkt::WktWriter;
    /// use crate::geozero::GeozeroGeometry;
    /// use crate::geozero::GeomProcessor;
    /// let input = GeoJson(r#"{ "type": "Point", "coordinates": [1.1, 1.2] }"#);
    ///
    /// let mut output = vec![] ;
    /// let mut wkt_writer = WktWriter::new(&mut output).pre_process_xy(|x: &mut f64, y: &mut f64| {
    ///    // likely you would do something more interesting here, like project your coordinates
    ///    *x += 1.0;
    ///    *y += 1.0;
    ///});
    ///
    /// input.process_geom(&mut wkt_writer).unwrap();
    /// assert_eq!(String::from_utf8(output).unwrap(), "POINT(2.1 2.2)");
    /// ```
    fn pre_process_xy<F: Fn(&mut f64, &mut f64)>(
        self,
        transform_xy: F,
    ) -> WrappedXYProcessor<Self, F>
    where
        Self: Sized,
    {
        WrappedXYProcessor::new(self, transform_xy)
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
