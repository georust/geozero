//! Traits for reading and creating geomeries.
//!
//! Main traits:
//! * [GeometryReader]: Reading geometries by passing events to a visitor object
//! * [GeomEventProcessor]: Processing geometry events, e.g. for producing an output geometry
//!
//! Main structs:
//! * [GeomVisitor]: Geometry visitor emitting events to a [GeomEventProcessor]
//!
//! ```md
//! Geometry --------------->    GeomVisitor
//!           GeometryReader  <GeomEventProcessor> -------------------> Geometry
//!                                                 GeomEventProcessor
//! ```
//
// GeomProcessor API:
// ```md
// Geometry ----------------> <GeomProcessor> -------------------> Geometry
//           GeozeroGeometry                     GeomProcessor
// ```

use crate::error::{GeozeroError, Result};
use crate::GeomProcessor;

/// Geometry processing events
///
/// State machine:
/// ```md
///                           +-----------------+
///                           |                 |
///                           |     Initial     <-------------+
///                           |                 |             |
///                           +--------^--------+    +--------v---------+
///                                    |             |                  |
///                              +-+-+-+-+-+         |GeometryCollection|
///                              | | | | | |         |                  |
///                              v v v v v v         +--------^---------+
/// +-----------------+                                       |
/// |                 |                                 +-+-+-+-+-+
/// |  MultiPolygon   |                                 | | | | | |
/// |                 |                                 v v v v v v
/// +--------^--------+   +-----------------+                         +-----------------+
///          |            |                 |                         |                 |
///          |            | MultiLineString |                         |   MultiPoint    |
/// +--------v--------+   |                 |                         |                 |
/// |                 |   +--------^--------+                         +--------^--------+
/// |     Polygon     |            |                                           |
/// |                 |            |                                           |
/// +--------^--------+   +--------v--------+                         +--------v--------+
///          |            |                 |                         |                 |
///          +----------->|    LineString   |                         |     Point       |
///                       |                 |                         |                 |
///                       +--------+--------+   +-----------------+   +--------+--------+
///                                |            |                 |            |
///                                +------------>   Coordinate    <------------+
///                                             |                 |
///                                             +-----------------+
/// ```
#[derive(Clone, PartialEq, Debug)]
pub enum Event {
    /// Coordinate with x,y dimensions (x, y, idx)
    Xy(f64, f64, usize),
    /// Coordinate with all requested dimensions (x, y, z, m, t, tm, idx)
    Coordinate(
        f64,
        f64,
        Option<f64>,
        Option<f64>,
        Option<f64>,
        Option<u64>,
        usize,
    ),
    /// Empty coordinates, like WKT's `POINT EMPTY` (idx)
    EmptyPoint(usize),
    /// Begin of Point (idx)
    PointBegin(usize),
    /// End of Point (idx)
    PointEnd(usize),
    /// Begin of MultiPoint (size, idx)
    MultiPointBegin(usize, usize),
    /// End of MultiPoint (idx)
    MultiPointEnd(usize),
    /// Begin of LineString (size, idx)
    ///
    /// Can be also a Polygon ring or part of a MultiLineString
    LineStringBegin(usize, usize),
    /// End of LineString (idx)
    LineStringEnd(usize),
    /// Begin of MultiLineString (size, idx)
    MultiLineStringBegin(usize, usize),
    /// End of MultiLineString (idx)
    MultiLineStringEnd(usize),
    /// Begin of Polygon (size, idx)
    PolygonBegin(usize, usize),
    /// End of Polygon (idx)
    PolygonEnd(usize),
    /// Begin of MultiPolygon (size, idx)
    MultiPolygonBegin(usize, usize),
    /// End of MultiPolygon (idx)
    MultiPolygonEnd(usize),
    /// Begin of GeometryCollection (size, idx)
    GeometryCollectionBegin(usize, usize),
    /// End of GeometryCollection (idx)
    GeometryCollectionEnd(usize),
    /// Begin of CircularString (size, idx)
    ///
    /// The CircularString is the basic curve type, similar to a LineString in the linear world. A single segment required three points, the start and end points (first and third) and any other point on the arc. The exception to this is for a closed circle, where the start and end points are the same. In this case the second point MUST be the center of the arc, ie the opposite side of the circle. To chain arcs together, the last point of the previous arc becomes the first point of the next arc, just like in LineString. This means that a valid circular string must have an odd number of points greated than 1.
    CircularStringBegin(usize, usize),
    /// End of CircularString (idx)
    CircularStringEnd(usize),
    /// Begin of CompoundCurve (size, idx)
    ///
    /// A compound curve is a single, continuous curve that has both curved (circular) segments and linear segments. That means that in addition to having well-formed components, the end point of every component (except the last) must be coincident with the start point of the following component.
    CompoundCurveBegin(usize, usize),
    /// End of CompoundCurve (idx)
    CompoundCurveEnd(usize),
    /// Begin of CurvePolygon (size, idx)
    ///
    /// A CurvePolygon is just like a polygon, with an outer ring and zero or more inner rings. The difference is that a ring can take the form of a circular string, linear string or compound string.
    CurvePolygonBegin(usize, usize),
    /// End of CurvePolygon (idx)
    CurvePolygonEnd(usize),
    /// Begin of MultiCurve (size, idx)
    ///
    /// The MultiCurve is a collection of curves, which can include linear strings, circular strings or compound strings.
    MultiCurveBegin(usize, usize),
    /// End of MultiCurve (idx)
    MultiCurveEnd(usize),
    /// Begin of MultiSurface (size, idx)
    ///
    /// The MultiSurface is a collection of surfaces, which can be (linear) polygons or curve polygons.
    MultiSurfaceBegin(usize, usize),
    /// End of MultiSurface (idx)
    MultiSurfaceEnd(usize),
    /// Begin of Triangle (size, idx)
    ///
    /// An untagged Triangle is part of a Tin
    TriangleBegin(usize, usize),
    /// End of Triangle (idx)
    TriangleEnd(usize),
    /// Begin of PolyhedralSurface (size, idx)
    PolyhedralSurfaceBegin(usize, usize),
    /// End of PolyhedralSurface (idx)
    PolyhedralSurfaceEnd(usize),
    /// Begin of Tin (size, idx)
    TinBegin(usize, usize),
    /// End of Tin (idx)
    TinEnd(usize),
}

/// Main Geometry type
///
/// This is the first state after `Initial` or `GeometryCollection`
/// according to the state diagram of [Event]
// WKB Types according to OGC 06-103r4 (<https://www.ogc.org/standards/sfa>)
#[derive(PartialEq, Copy, Clone, Debug)]
pub enum GeometryType {
    Unknown,
    Point,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    //GeometryCollection,
    CircularString,
    CompoundCurve,
    CurvePolygon,
    MultiCurve,
    MultiSurface,
    Curve,
    Surface,
    PolyhedralSurface,
    Tin,
    Triangle,
}

/// Geometry visitor emitting events to a processor
pub struct GeomVisitor<'a, P: GeomEventProcessor> {
    // pub dims: CoordDimensions,
    /// Main geometry type
    geom_type: GeometryType,
    /// Geometry is part of collection
    collection: bool,
    processor: &'a mut P,
}

/// Processing geometry events, e.g. for producing an output geometry
pub trait GeomEventProcessor {
    /// Geometry processing event with geometry type information
    fn event(&mut self, event: Event, geom_type: GeometryType, collection: bool) -> Result<()>;
}

/// Geometry processor without any actions
pub struct GeomEventSink;

impl GeomEventProcessor for GeomEventSink {
    fn event(&mut self, _event: Event, _geom_type: GeometryType, _collection: bool) -> Result<()> {
        Ok(())
    }
}

/// Reading geometries by passing events to a visitor object
pub trait GeometryReader {
    /// Process geometry.
    fn process_geom<P: GeomEventProcessor>(
        &mut self,
        visitor: &mut GeomVisitor<'_, P>,
    ) -> Result<()>;
}

impl<'a, P: GeomEventProcessor> GeomVisitor<'a, P> {
    pub fn new(processor: &'a mut P) -> Self {
        GeomVisitor {
            geom_type: GeometryType::Unknown,
            collection: false,
            processor,
        }
    }
    pub fn emit(&mut self, event: Event) -> Result<()> {
        self.processor.event(event, self.geom_type, self.collection)
    }
    fn set_type(&mut self, inner_type: GeometryType) -> Result<()> {
        match self.geom_type {
            GeometryType::Unknown => {
                self.geom_type = inner_type;
            }
            _ if self.collection => {
                // new type within collection
                self.geom_type = inner_type;
            }
            _ => {
                // type already defined (check if self.geom_type = match inner_type ?)
            }
        }
        Ok(())
    }
    fn check_type(&self, geom_type: GeometryType) -> Result<()> {
        if self.geom_type == geom_type {
            Ok(())
        } else {
            self.type_seq_err()
        }
    }
    fn type_seq_err(&self) -> Result<()> {
        // TODO: unexpected event in self.geom_type
        Err(GeozeroError::GeometryFormat)
    }

    /// Process coordinate with x,y dimensions
    pub fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.emit(Event::Xy(x, y, idx))?;
        Ok(())
    }

    /// Process coordinate with all requested dimensions
    pub fn coordinate(
        &mut self,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
        idx: usize,
    ) -> Result<()> {
        self.emit(Event::Coordinate(x, y, z, m, t, tm, idx))?;
        Ok(())
    }
    /// Begin of Point processing
    pub fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.set_type(GeometryType::Point)?;
        self.emit(Event::PointBegin(idx))?;
        Ok(())
    }

    /// End of Point processing
    pub fn point_end(&mut self, idx: usize) -> Result<()> {
        // self.check_type(GeometryType::Point)?; || MultiPoint, etc.
        self.emit(Event::PointEnd(idx))?;
        Ok(())
    }

    pub fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::MultiPoint)?;
        self.emit(Event::MultiPointBegin(size, idx))?;
        Ok(())
    }

    /// End of MultiPoint processing
    pub fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.check_type(GeometryType::MultiPoint)?;
        self.emit(Event::MultiPointEnd(idx))?;
        Ok(())
    }

    /// Begin of LineString processing
    pub fn linestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::LineString)?;
        self.emit(Event::LineStringBegin(size, idx))?;
        Ok(())
    }

    /// End of LineString processing
    pub fn linestring_end(&mut self, idx: usize) -> Result<()> {
        // self.check_type(GeometryType::LineString)?; || polygon, etc.
        self.emit(Event::LineStringEnd(idx))?;
        Ok(())
    }

    /// Begin of Polygon processing
    pub fn polygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::Polygon)?;
        self.emit(Event::PolygonBegin(size, idx))?;
        Ok(())
    }

    /// End of Polygon processing
    pub fn polygon_end(&mut self, idx: usize) -> Result<()> {
        self.check_type(GeometryType::Polygon)?;
        self.emit(Event::PolygonEnd(idx))?;
        Ok(())
    }

    /// Begin of GeometryCollection processing
    pub fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.collection = true;
        self.geom_type = GeometryType::Unknown;
        self.emit(Event::GeometryCollectionBegin(size, idx))?;
        Ok(())
    }

    /// End of GeometryCollection processing
    pub fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        if !self.collection {
            return self.type_seq_err();
        }
        self.geom_type = GeometryType::Unknown;
        self.emit(Event::GeometryCollectionEnd(idx))?;
        self.collection = false;
        Ok(())
    }
}

#[allow(unused)]
impl<'a, P: GeomEventProcessor> GeomProcessor for GeomVisitor<'a, P> {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.xy(x, y, idx)
    }
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
        self.coordinate(x, y, z, m, t, tm, idx)
    }
    fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.point_begin(idx)?;
        self.point_end(idx)
    }
    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.point_begin(idx)
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.point_end(idx)
    }
    fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.multipoint_begin(size, idx)
    }
    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.multipoint_end(idx)
    }
    fn linestring_begin(&mut self, _tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.linestring_begin(size, idx)
    }
    fn linestring_end(&mut self, _tagged: bool, idx: usize) -> Result<()> {
        self.linestring_end(idx)
    }
    fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn polygon_begin(&mut self, _tagged: bool, size: usize, idx: usize) -> Result<()> {
        self.polygon_begin(size, idx)
    }
    fn polygon_end(&mut self, _tagged: bool, idx: usize) -> Result<()> {
        self.polygon_end(idx)
    }
    fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.geometrycollection_begin(size, idx)
    }
    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.geometrycollection_end(idx)
    }
    fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()> {
        todo!()
    }
    fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
    fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        todo!()
    }
    fn tin_end(&mut self, idx: usize) -> Result<()> {
        todo!()
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::events::Event::*;

    // -- Event emitter (geometry input) --

    pub struct NullIsland;

    impl GeometryReader for NullIsland {
        fn process_geom<P: GeomEventProcessor>(
            &mut self,
            visitor: &mut GeomVisitor<'_, P>,
        ) -> Result<()> {
            visitor.point_begin(0)?;
            visitor.xy(0.0, 0.0, 0)?;
            visitor.point_end(0)?;
            Ok(())
        }
    }

    // -- Event processor (geometry output) --

    pub struct GeomEventBuffer {
        pub buffer: Vec<Event>,
    }

    impl GeomEventBuffer {
        pub fn new() -> Self {
            GeomEventBuffer { buffer: Vec::new() }
        }
    }

    impl GeomEventProcessor for GeomEventBuffer {
        fn event(
            &mut self,
            event: Event,
            _geom_type: GeometryType,
            _collection: bool,
        ) -> Result<()> {
            self.buffer.push(event);
            Ok(())
        }
    }

    #[test]
    fn processing() -> Result<()> {
        let mut processor = GeomEventBuffer::new();
        let mut visitor = GeomVisitor::new(&mut processor);

        let mut geom = NullIsland;
        geom.process_geom(&mut visitor)?;

        assert_eq!(
            processor.buffer,
            [PointBegin(0), Xy(0.0, 0.0, 0), PointEnd(0)]
        );

        Ok(())
    }

    struct Point2D {
        pub x: f64,
        pub y: f64,
    }

    impl GeomEventProcessor for Point2D {
        fn event(
            &mut self,
            event: Event,
            _geom_type: GeometryType,
            _collection: bool,
        ) -> Result<()> {
            match event {
                PointBegin(_) | PointEnd(_) => {} // OK
                Xy(x, y, _idx) => (self.x, self.y) = (x, y),
                _ => return Err(GeozeroError::GeometryFormat),
            }
            Ok(())
        }
    }

    #[test]
    fn process_point() -> Result<()> {
        let mut geom_out = Point2D {
            x: f64::NAN,
            y: f64::NAN,
        };
        let mut visitor = GeomVisitor::new(&mut geom_out);

        let mut geom_in = NullIsland;
        geom_in.process_geom(&mut visitor)?;

        assert_eq!((geom_out.x, geom_out.y), (0.0, 0.0));

        Ok(())
    }

    #[test]
    fn polygon() -> Result<()> {
        let mut processor = GeomEventBuffer::new();
        let mut visitor = GeomVisitor::new(&mut processor);
        visitor.polygon_begin(2, 0)?;
        visitor.linestring_begin(2, 0)?;
        visitor.xy(0.0, 0.0, 0)?;
        visitor.xy(1.0, 1.0, 1)?;
        visitor.linestring_end(0)?;
        visitor.linestring_begin(2, 1)?;
        visitor.xy(0.0, 0.0, 0)?;
        visitor.xy(1.0, 1.0, 1)?;
        visitor.linestring_end(1)?;
        visitor.polygon_end(0)?;

        dbg!(&processor.buffer);
        assert_eq!(
            processor.buffer,
            [
                PolygonBegin(2, 0),
                LineStringBegin(2, 0),
                Xy(0.0, 0.0, 0),
                Xy(1.0, 1.0, 1),
                LineStringEnd(0),
                LineStringBegin(2, 1),
                Xy(0.0, 0.0, 0),
                Xy(1.0, 1.0, 1),
                LineStringEnd(1),
                PolygonEnd(0)
            ]
        );

        Ok(())
    }

    #[test]
    fn collection() -> Result<()> {
        let mut processor = GeomEventBuffer::new();
        let mut visitor = GeomVisitor::new(&mut processor);
        visitor.geometrycollection_begin(2, 0)?;
        visitor.point_begin(0)?;
        visitor.xy(0.0, 0.0, 0)?;
        visitor.point_end(0)?;
        visitor.linestring_begin(2, 1)?;
        visitor.xy(0.0, 0.0, 0)?;
        visitor.xy(1.0, 1.0, 1)?;
        visitor.linestring_end(1)?;
        visitor.geometrycollection_end(0)?;

        assert_eq!(
            processor.buffer,
            [
                GeometryCollectionBegin(2, 0),
                PointBegin(0),
                Xy(0.0, 0.0, 0),
                PointEnd(0),
                LineStringBegin(2, 1),
                Xy(0.0, 0.0, 0),
                Xy(1.0, 1.0, 1),
                LineStringEnd(1),
                GeometryCollectionEnd(0)
            ]
        );

        Ok(())
    }

    #[test]
    fn invalid_transitions() -> Result<()> {
        let mut processor = GeomEventSink {};
        let mut visitor = GeomVisitor::new(&mut processor);
        visitor.point_begin(0)?;
        visitor.xy(0.0, 0.0, 0)?;
        let result = visitor.polygon_end(0);
        assert!(result.is_err());

        Ok(())
    }

    #[test]
    #[cfg(feature = "with-geojson")]
    fn geozero_geometry_api() -> Result<()> {
        use crate::api::GeozeroGeometry;
        use crate::geojson::GeoJson;

        let geojson = GeoJson(
            r#"{"type": "Feature", "properties": {"fid": 0, "name": "Albania"}, "geometry": {"type": "Polygon", "coordinates": [[[20.590247,41.855404],[20.463175,41.515089],[20.605182,41.086226],[21.02004,40.842727],[20.99999,40.580004],[20.674997,40.435],[20.615,40.110007],[20.150016,39.624998],[19.98,39.694993],[19.960002,39.915006],[19.406082,40.250773],[19.319059,40.72723],[19.40355,41.409566],[19.540027,41.719986],[19.371769,41.877548],[19.304486,42.195745],[19.738051,42.688247],[19.801613,42.500093],[20.0707,42.58863],[20.283755,42.32026],[20.52295,42.21787],[20.590247,41.855404]]]}}"#,
        );
        let mut processor = GeomEventBuffer::new();
        geojson.process_geom(&mut GeomVisitor::new(&mut processor))?;
        assert_eq!(
            processor.buffer,
            [
                PolygonBegin(1, 0),
                LineStringBegin(22, 0),
                Xy(20.590247, 41.855404, 0),
                Xy(20.463175, 41.515089, 1),
                Xy(20.605182, 41.086226, 2),
                Xy(21.02004, 40.842727, 3),
                Xy(20.99999, 40.580004, 4),
                Xy(20.674997, 40.435, 5),
                Xy(20.615, 40.110007, 6),
                Xy(20.150016, 39.624998, 7),
                Xy(19.98, 39.694993, 8),
                Xy(19.960002, 39.915006, 9),
                Xy(19.406082, 40.250773, 10),
                Xy(19.319059, 40.72723, 11),
                Xy(19.40355, 41.409566, 12),
                Xy(19.540027, 41.719986, 13),
                Xy(19.371769, 41.877548, 14),
                Xy(19.304486, 42.195745, 15),
                Xy(19.738051, 42.688247, 16),
                Xy(19.801613, 42.500093, 17),
                Xy(20.0707, 42.58863, 18),
                Xy(20.283755, 42.32026, 19),
                Xy(20.52295, 42.21787, 20),
                Xy(20.590247, 41.855404, 21),
                LineStringEnd(0),
                PolygonEnd(0)
            ]
        );
        Ok(())
    }
}
