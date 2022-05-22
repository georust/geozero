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
///                       +--------+--------+                         +--------+--------+
///                                |                                           |
///                                +------------>   Coordinate    <------------+
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

#[derive(PartialEq, Debug)]
enum Vstate {
    Initial,
    Point,
    LineString,
    Polygon,
    MultiPoint,
    MultiLineString,
    MultiPolygon,
    GeometryCollection,
    CircularString,
    CompoundCurve,
    CurvePolygon,
    MultiCurve,
    MultiSurface,
    #[allow(dead_code)]
    Curve,
    #[allow(dead_code)]
    Surface,
    PolyhedralSurface,
    Tin,
    Triangle,
}

/// Geometry visitor emitting events to a processor
pub struct GeomVisitor<'a, P: GeomEventProcessor> {
    // pub dims: CoordDimensions,
    pub check_states: bool,
    /// Main geometry type
    geom_type: GeometryType,
    /// Geometry is part of collection
    collection: bool,
    state_stack: Vec<Vstate>,
    processor: &'a mut P,
}

/// Processing geometry events, e.g. for producing an output geometry
pub trait GeomEventProcessor {
    /// Geometry processing event with geometry type information
    fn event(&mut self, event: Event, geom_type: GeometryType, collection: bool) -> Result<()>;
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
            check_states: true, // Should maybe set from env var?
            geom_type: GeometryType::Unknown,
            collection: false,
            state_stack: Vec::new(),
            processor,
        }
    }
    fn emit(&mut self, event: Event) -> Result<()> {
        self.processor.event(event, self.geom_type, self.collection)
    }
    /// Pass event to chained visitor with original state
    pub fn chain_event(
        &mut self,
        event: Event,
        geom_type: GeometryType,
        collection: bool,
    ) -> Result<()> {
        self.processor.event(event, geom_type, collection)
    }
    /// Pass event to visitor with state recalculation
    pub fn emit_event(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Xy(x, y, idx) => self.xy(x, y, idx),
            Event::Coordinate(x, y, z, m, t, tm, idx) => self.coordinate(x, y, z, m, t, tm, idx),
            Event::EmptyPoint(idx) => self.empty_point(idx),
            Event::PointBegin(idx) => self.point_begin(idx),
            Event::PointEnd(idx) => self.point_end(idx),
            Event::MultiPointBegin(size, idx) => self.multipoint_begin(size, idx),
            Event::MultiPointEnd(idx) => self.multipoint_end(idx),
            Event::LineStringBegin(size, idx) => self.linestring_begin(size, idx),
            Event::LineStringEnd(idx) => self.linestring_end(idx),
            Event::MultiLineStringBegin(size, idx) => self.multilinestring_begin(size, idx),
            Event::MultiLineStringEnd(idx) => self.multilinestring_end(idx),
            Event::PolygonBegin(size, idx) => self.polygon_begin(size, idx),
            Event::PolygonEnd(idx) => self.polygon_end(idx),
            Event::MultiPolygonBegin(size, idx) => self.multipolygon_begin(size, idx),
            Event::MultiPolygonEnd(idx) => self.multipolygon_end(idx),
            Event::GeometryCollectionBegin(size, idx) => self.geometrycollection_begin(size, idx),
            Event::GeometryCollectionEnd(idx) => self.geometrycollection_end(idx),
            Event::CircularStringBegin(size, idx) => self.circularstring_begin(size, idx),
            Event::CircularStringEnd(idx) => self.circularstring_end(idx),
            Event::CompoundCurveBegin(size, idx) => self.compoundcurve_begin(size, idx),
            Event::CompoundCurveEnd(idx) => self.compoundcurve_end(idx),
            Event::CurvePolygonBegin(size, idx) => self.curvepolygon_begin(size, idx),
            Event::CurvePolygonEnd(idx) => self.curvepolygon_end(idx),
            Event::MultiCurveBegin(size, idx) => self.multicurve_begin(size, idx),
            Event::MultiCurveEnd(idx) => self.multicurve_end(idx),
            Event::MultiSurfaceBegin(size, idx) => self.multisurface_begin(size, idx),
            Event::MultiSurfaceEnd(idx) => self.multisurface_end(idx),
            Event::TriangleBegin(size, idx) => self.triangle_begin(size, idx),
            Event::TriangleEnd(idx) => self.triangle_end(idx),
            Event::PolyhedralSurfaceBegin(size, idx) => self.polyhedralsurface_begin(size, idx),
            Event::PolyhedralSurfaceEnd(idx) => self.polyhedralsurface_end(idx),
            Event::TinBegin(size, idx) => self.tin_begin(size, idx),
            Event::TinEnd(idx) => self.tin_end(idx),
        }
    }
    fn state(&self) -> &Vstate {
        let len = self.state_stack.len();
        if len > 0 {
            &self.state_stack[len - 1]
        } else {
            &Vstate::Initial
        }
    }
    fn enter_state(&mut self, state: Vstate) -> Result<()> {
        match (self.state(), &state) {
            (Vstate::Initial, Vstate::GeometryCollection)
            | (Vstate::Initial, Vstate::Point)
            | (Vstate::Initial, Vstate::LineString)
            | (Vstate::Initial, Vstate::Polygon)
            | (Vstate::Initial, Vstate::MultiPoint)
            | (Vstate::Initial, Vstate::MultiLineString)
            | (Vstate::Initial, Vstate::MultiPolygon)
            | (Vstate::Initial, Vstate::CircularString)
            | (Vstate::Initial, Vstate::CompoundCurve)
            | (Vstate::Initial, Vstate::CurvePolygon)
            | (Vstate::Initial, Vstate::MultiCurve)
            | (Vstate::Initial, Vstate::MultiSurface)
            | (Vstate::Initial, Vstate::Curve)
            | (Vstate::Initial, Vstate::Surface)
            | (Vstate::Initial, Vstate::PolyhedralSurface)
            | (Vstate::Initial, Vstate::Tin)
            | (Vstate::Initial, Vstate::Triangle)
            | (Vstate::Polygon, Vstate::LineString)
            | (Vstate::MultiPoint, Vstate::Point)
            | (Vstate::MultiLineString, Vstate::LineString)
            | (Vstate::MultiPolygon, Vstate::Polygon)
            | (Vstate::GeometryCollection, Vstate::Point)
            | (Vstate::GeometryCollection, Vstate::LineString)
            | (Vstate::GeometryCollection, Vstate::Polygon)
            | (Vstate::GeometryCollection, Vstate::MultiPoint)
            | (Vstate::GeometryCollection, Vstate::MultiLineString)
            | (Vstate::GeometryCollection, Vstate::MultiPolygon)
            | (Vstate::GeometryCollection, Vstate::GeometryCollection)
            | (Vstate::GeometryCollection, Vstate::CircularString)
            | (Vstate::GeometryCollection, Vstate::CompoundCurve)
            | (Vstate::GeometryCollection, Vstate::CurvePolygon)
            | (Vstate::GeometryCollection, Vstate::MultiCurve)
            | (Vstate::GeometryCollection, Vstate::MultiSurface)
            | (Vstate::GeometryCollection, Vstate::Curve)
            | (Vstate::GeometryCollection, Vstate::Surface)
            | (Vstate::GeometryCollection, Vstate::PolyhedralSurface)
            | (Vstate::GeometryCollection, Vstate::Tin)
            | (Vstate::GeometryCollection, Vstate::Triangle)
            | (Vstate::CompoundCurve, Vstate::CircularString)
            | (Vstate::CompoundCurve, Vstate::LineString)
            | (Vstate::CurvePolygon, Vstate::CircularString)
            | (Vstate::CurvePolygon, Vstate::LineString)
            | (Vstate::CurvePolygon, Vstate::CompoundCurve)
            | (Vstate::MultiCurve, Vstate::CircularString)
            | (Vstate::MultiCurve, Vstate::LineString)
            | (Vstate::MultiCurve, Vstate::CompoundCurve)
            | (Vstate::MultiSurface, Vstate::CurvePolygon)
            | (Vstate::MultiSurface, Vstate::Polygon)
            | (Vstate::Triangle, Vstate::LineString)
            | (Vstate::PolyhedralSurface, Vstate::Polygon)
            | (Vstate::Tin, Vstate::Polygon) => {
                // println!("Enter state {:?}=>{:?}", self.state(), state);
                self.state_stack.push(state);
                Ok(())
            }
            _ => Err(GeozeroError::Geometry(format!(
                "Invalid state transition from {:?} to {:?}",
                self.state(),
                state
            ))),
        }
    }
    fn exit_state(&mut self, state: Vstate) -> Result<()> {
        let valid = if let Some(prev_state) = self.state_stack.pop() {
            state == prev_state
        } else {
            false
        };
        if valid {
            // println!(
            //     "Exit state {:?} (GeometryType::{:?})=>{:?}",
            //     &self.state_stack,
            //     &self.geom_type,
            //     self.state()
            // );
        } else {
            return Err(GeozeroError::Geometry(format!(
                "Invalid state transition from {:?} to {:?}",
                self.state_stack, state
            )));
        }
        Ok(())
    }
    fn set_type(&mut self, inner_type: GeometryType) -> Result<()> {
        if self.geom_type == GeometryType::Unknown {
            // println!("Set GeometryType {:?} => {:?}", &self.geom_type, inner_type);
            self.geom_type = inner_type;
        }
        Ok(())
    }
    fn reset_type(&mut self, inner_type: GeometryType) {
        // Reset geometry type within collections
        if self.collection && self.geom_type == inner_type {
            self.geom_type = GeometryType::Unknown;
        }
    }
    /// Process coordinate with x,y dimensions
    pub fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.emit(Event::Xy(x, y, idx))
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
        self.emit(Event::Coordinate(x, y, z, m, t, tm, idx))
    }
    /// Process empty coordinates, like WKT's `POINT EMPTY`
    pub fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.set_type(GeometryType::Point)?;
        if self.check_states {
            self.enter_state(Vstate::Point)?;
        }
        self.emit(Event::EmptyPoint(idx))?;
        if self.check_states {
            self.exit_state(Vstate::Point)?;
        }
        Ok(())
    }

    /// Begin of Point processing
    pub fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.set_type(GeometryType::Point)?;
        if self.check_states {
            self.enter_state(Vstate::Point)?;
        }
        self.emit(Event::PointBegin(idx))?;
        Ok(())
    }

    /// End of Point processing
    pub fn point_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::PointEnd(idx))?;
        self.reset_type(GeometryType::Point);
        if self.check_states {
            self.exit_state(Vstate::Point)?;
        }
        Ok(())
    }

    /// Begin of MultiPoint processing
    pub fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::MultiPoint)?;
        if self.check_states {
            self.enter_state(Vstate::MultiPoint)?;
        }
        self.emit(Event::MultiPointBegin(size, idx))?;
        Ok(())
    }

    /// End of MultiPoint processing
    pub fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::MultiPointEnd(idx))?;
        self.reset_type(GeometryType::MultiPoint);
        if self.check_states {
            self.exit_state(Vstate::MultiPoint)?;
        }
        Ok(())
    }

    /// Begin of LineString processing
    ///
    /// Can also be a Polygon ring or part of a MultiLineString
    ///
    /// Next: size * xy/coordinate
    pub fn linestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::LineString)?;
        if self.check_states {
            self.enter_state(Vstate::LineString)?;
        }
        self.emit(Event::LineStringBegin(size, idx))?;
        Ok(())
    }

    /// End of LineString processing
    pub fn linestring_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::LineStringEnd(idx))?;
        self.reset_type(GeometryType::LineString);
        if self.check_states {
            self.exit_state(Vstate::LineString)?;
        }
        Ok(())
    }

    /// Begin of MultiLineString processing
    ///
    /// Next: size * LineString
    pub fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::MultiLineString)?;
        if self.check_states {
            self.enter_state(Vstate::MultiLineString)?;
        }
        self.emit(Event::MultiLineStringBegin(size, idx))?;
        Ok(())
    }

    /// End of MultiLineString processing
    pub fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::MultiLineStringEnd(idx))?;
        self.reset_type(GeometryType::MultiLineString);
        if self.check_states {
            self.exit_state(Vstate::MultiLineString)?;
        }
        Ok(())
    }

    /// Begin of Polygon processing
    ///
    /// Can also be part of a MultiPolygon
    ///
    /// Next: size * LineString = rings
    pub fn polygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::Polygon)?;
        if self.check_states {
            self.enter_state(Vstate::Polygon)?;
        }
        self.emit(Event::PolygonBegin(size, idx))?;
        Ok(())
    }

    /// End of Polygon processing
    pub fn polygon_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::PolygonEnd(idx))?;
        self.reset_type(GeometryType::Polygon);
        if self.check_states {
            self.exit_state(Vstate::Polygon)?;
        }
        Ok(())
    }

    /// Begin of MultiPolygon processing
    ///
    /// Next: size * Polygon
    pub fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::MultiPolygon)?;
        if self.check_states {
            self.enter_state(Vstate::MultiPolygon)?;
        }
        self.emit(Event::MultiPolygonBegin(size, idx))?;
        Ok(())
    }

    /// End of MultiPolygon processing
    pub fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::MultiPolygonEnd(idx))?;
        self.reset_type(GeometryType::MultiPolygon);
        if self.check_states {
            self.exit_state(Vstate::MultiPolygon)?;
        }
        Ok(())
    }

    /// Begin of GeometryCollection processing
    pub fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.collection = true;
        self.geom_type = GeometryType::Unknown;
        if self.check_states {
            self.enter_state(Vstate::GeometryCollection)?;
        }
        self.emit(Event::GeometryCollectionBegin(size, idx))?;
        Ok(())
    }

    /// End of GeometryCollection processing
    pub fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::GeometryCollectionEnd(idx))?;
        self.geom_type = GeometryType::Unknown;
        if self.check_states {
            self.exit_state(Vstate::GeometryCollection)?;
        }
        self.collection = false;
        Ok(())
    }

    /// Begin of CircularString processing
    ///
    /// The CircularString is the basic curve type, similar to a LineString in the linear world. A single segment required three points, the start and end points (first and third) and any other point on the arc. The exception to this is for a closed circle, where the start and end points are the same. In this case the second point MUST be the center of the arc, ie the opposite side of the circle. To chain arcs together, the last point of the previous arc becomes the first point of the next arc, just like in LineString. This means that a valid circular string must have an odd number of points greated than 1.
    ///
    /// Next: size * xy/coordinate
    pub fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::CircularString)?;
        if self.check_states {
            self.enter_state(Vstate::CircularString)?;
        }
        self.emit(Event::CircularStringBegin(size, idx))?;
        Ok(())
    }

    /// End of CircularString processing
    pub fn circularstring_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::CircularStringEnd(idx))?;
        self.reset_type(GeometryType::CircularString);
        if self.check_states {
            self.exit_state(Vstate::CircularString)?;
        }
        Ok(())
    }

    /// Begin of CompoundCurve processing
    ///
    /// A compound curve is a single, continuous curve that has both curved (circular) segments and linear segments. That means that in addition to having well-formed components, the end point of every component (except the last) must be coincident with the start point of the following component.
    ///
    /// Next: size * (CircularString | LineString)
    pub fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::CompoundCurve)?;
        if self.check_states {
            self.enter_state(Vstate::CompoundCurve)?;
        }
        self.emit(Event::CompoundCurveBegin(size, idx))?;
        Ok(())
    }

    /// End of CompoundCurve processing
    pub fn compoundcurve_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::CompoundCurveEnd(idx))?;
        self.reset_type(GeometryType::CompoundCurve);
        if self.check_states {
            self.exit_state(Vstate::CompoundCurve)?;
        }
        Ok(())
    }

    /// Begin of CurvePolygon processing
    ///
    /// A CurvePolygon is just like a polygon, with an outer ring and zero or more inner rings. The difference is that a ring can take the form of a circular string, linear string or compound string.
    ///
    /// Next: size * (CircularString | LineString | CompoundCurve)
    pub fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::CurvePolygon)?;
        if self.check_states {
            self.enter_state(Vstate::CurvePolygon)?;
        }
        self.emit(Event::CurvePolygonBegin(size, idx))?;
        Ok(())
    }

    /// End of CurvePolygon processing
    pub fn curvepolygon_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::CurvePolygonEnd(idx))?;
        self.reset_type(GeometryType::CurvePolygon);
        if self.check_states {
            self.exit_state(Vstate::CurvePolygon)?;
        }
        Ok(())
    }

    /// Begin of MultiCurve processing
    ///
    /// The MultiCurve is a collection of curves, which can include linear strings, circular strings or compound strings.
    ///
    /// Next: size * (CircularString | LineString | CompoundCurve)
    pub fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::MultiCurve)?;
        if self.check_states {
            self.enter_state(Vstate::MultiCurve)?;
        }
        self.emit(Event::MultiCurveBegin(size, idx))?;
        Ok(())
    }

    /// End of MultiCurve processing
    pub fn multicurve_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::MultiCurveEnd(idx))?;
        self.reset_type(GeometryType::MultiCurve);
        if self.check_states {
            self.exit_state(Vstate::MultiCurve)?;
        }
        Ok(())
    }

    /// Begin of MultiSurface processing
    ///
    /// The MultiSurface is a collection of surfaces, which can be (linear) polygons or curve polygons.
    ///
    /// Next: size * (CurvePolygon | Polygon)
    pub fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::MultiSurface)?;
        if self.check_states {
            self.enter_state(Vstate::MultiSurface)?;
        }
        self.emit(Event::MultiSurfaceBegin(size, idx))?;
        Ok(())
    }

    /// End of MultiSurface processing
    pub fn multisurface_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::MultiSurfaceEnd(idx))?;
        self.reset_type(GeometryType::MultiSurface);
        if self.check_states {
            self.exit_state(Vstate::MultiSurface)?;
        }
        Ok(())
    }
    /// Begin of Triangle processing
    ///
    /// Can also be part of a Tin
    ///
    /// Next: size * LineString = rings
    pub fn triangle_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::Triangle)?;
        if self.check_states {
            self.enter_state(Vstate::Triangle)?;
        }
        self.emit(Event::TriangleBegin(size, idx))?;
        Ok(())
    }

    /// End of Triangle processing
    pub fn triangle_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::TriangleEnd(idx))?;
        self.reset_type(GeometryType::Triangle);
        if self.check_states {
            self.exit_state(Vstate::Triangle)?;
        }
        Ok(())
    }

    /// Begin of PolyhedralSurface processing
    ///
    /// Next: size * Polygon
    pub fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::PolyhedralSurface)?;
        if self.check_states {
            self.enter_state(Vstate::PolyhedralSurface)?;
        }
        self.emit(Event::PolyhedralSurfaceBegin(size, idx))?;
        Ok(())
    }

    /// End of PolyhedralSurface processing
    pub fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::PolyhedralSurfaceEnd(idx))?;
        self.reset_type(GeometryType::PolyhedralSurface);
        if self.check_states {
            self.exit_state(Vstate::PolyhedralSurface)?;
        }
        Ok(())
    }

    /// Begin of Tin processing
    ///
    /// Next: size * Polygon
    pub fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()> {
        self.set_type(GeometryType::Tin)?;
        if self.check_states {
            self.enter_state(Vstate::Tin)?;
        }
        self.emit(Event::TinBegin(size, idx))?;
        Ok(())
    }

    /// End of Tin processing
    pub fn tin_end(&mut self, idx: usize) -> Result<()> {
        self.emit(Event::TinEnd(idx))?;
        self.reset_type(GeometryType::Tin);
        if self.check_states {
            self.exit_state(Vstate::Tin)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod test {
    use super::*;
    use crate::events::Event::*;
    use crate::processor::GeomEventSink;

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
        let mut processor = GeomEventSink;
        let mut visitor = GeomVisitor::new(&mut processor);
        visitor.point_begin(0)?;
        visitor.xy(0.0, 0.0, 0)?;
        let result = visitor.polygon_end(0);
        assert!(result.is_err());

        visitor.check_states = false;
        visitor.point_begin(0)?;
        visitor.xy(0.0, 0.0, 0)?;
        visitor.polygon_end(0)?;

        Ok(())
    }

    #[test]
    #[cfg(feature = "with-geojson")]
    fn geozero_geometry_api() -> Result<()> {
        use crate::api::GeozeroGeometry;
        use crate::geojson::GeoJson;

        let geojson = GeoJson(
            r#"{"type": "Polygon", "coordinates": [[[20.590247,41.855404],[20.463175,41.515089],[20.605182,41.086226],[21.02004,40.842727],[20.99999,40.580004],[20.674997,40.435],[20.615,40.110007],[20.150016,39.624998],[19.98,39.694993],[19.960002,39.915006],[19.406082,40.250773],[19.319059,40.72723],[19.40355,41.409566],[19.540027,41.719986],[19.371769,41.877548],[19.304486,42.195745],[19.738051,42.688247],[19.801613,42.500093],[20.0707,42.58863],[20.283755,42.32026],[20.52295,42.21787],[20.590247,41.855404]]]}"#,
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
