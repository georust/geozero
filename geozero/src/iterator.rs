//! Event emitting geometry iterator.
// Inspiration: https://docs.rs/lyon_path/latest/lyon_path/enum.Event.html#

use crate::error::{GeozeroError, Result};
use crate::events::{GeometryType, Vstate};
use crate::CoordDimensions;

pub struct EventIter<T> {
    visitor: T,
    state: IterState,
}

struct IterState {
    /// Main geometry type
    pub geom_type: GeometryType,
    /// Geometry is part of collection
    pub collection: bool,
    // Iterator settings
    iter_dims: CoordDimensions,
    check_states: bool,
    state_stack: Vec<Vstate>,
    // Convert single to multi geometries, if declared as multi type or Unknown
    promote_to_multi: bool,
}

impl IterState {
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
}

impl<T> EventIter<T> {
    // Default iterator (XY)
    pub fn new(visitor: T) -> EventIter<T> {
        Self::new_with_config(visitor, CoordDimensions::xy(), true, false)
    }
    pub fn new_with_config(
        visitor: T,
        dims: CoordDimensions,
        check_states: bool,
        promote_to_multi: bool,
    ) -> EventIter<T> {
        EventIter::<T> {
            visitor,
            state: IterState {
                geom_type: GeometryType::Unknown,
                collection: false,
                iter_dims: dims,
                check_states,
                state_stack: Vec::new(),
                promote_to_multi,
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::events::Event;
    use std::io::{BufWriter, Write};

    pub struct NullIsland {
        events: Vec<Event>,
    }

    impl NullIsland {
        pub fn new() -> Self {
            Self {
                events: vec![
                    Event::PointBegin(1),
                    Event::Xy(0.0, 0.0, 0),
                    Event::PointEnd(1),
                ],
            }
        }
    }

    pub struct NullIslandVisitor<'a> {
        events: std::slice::Iter<'a, Event>,
    }

    impl<'a> NullIslandVisitor<'a> {
        pub fn new(geom: &'a NullIsland) -> Self {
            NullIslandVisitor {
                events: geom.events.iter(),
            }
        }
    }

    // Implement reader for NullIsland type
    impl<'a> Iterator for EventIter<NullIslandVisitor<'a>> {
        type Item = Result<&'a Event>;
        fn next(&mut self) -> Option<Self::Item> {
            // We have access to self.visitor and self.state
            if self.state.geom_type == GeometryType::Unknown {
                self.state.set_type(GeometryType::Point).unwrap(); //TODO: propagate errors
            }
            // we should also call enter_state/exit_state to ensure
            // valid state transitions.
            self.visitor.events.next().map(|ev| Ok(ev))
        }
    }

    #[test]
    fn null_island_to_wkt() -> std::io::Result<()> {
        let mut out = BufWriter::new(Vec::new());
        let geom = NullIsland::new();
        let visitor = NullIslandVisitor::new(&geom);
        for (i, event) in EventIter::new(visitor).enumerate() {
            let event = event.unwrap();
            assert_eq!(*event, geom.events[i]);
            let _ = match event {
                Event::PointBegin(_) => out.write(b"POINT(")?,
                Event::Xy(x, y, _) => out.write(&format!("{x} {y}").as_bytes())?,
                Event::PointEnd(_) => out.write(b")")?,
                _ => 0,
            };
        }
        assert_eq!(std::str::from_utf8(&out.into_inner()?), Ok("POINT(0 0)"));
        Ok(())
    }
}
