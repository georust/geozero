use crate::error::{GeozeroError, Result};
use std::io;

/// GPX reader
pub struct GpxReader<'a, R: io::Read>(pub &'a mut R);
pub struct Gpx<'a>(pub &'a str);

impl<'a, R: io::Read> crate::GeozeroDatasource for GpxReader<'a, R> {
    fn process<P: crate::FeatureProcessor>(
        &mut self,
        processor: &mut P,
    ) -> crate::error::Result<()> {
        read_gpx(&mut self.0, processor)
    }
}

impl<'a> crate::GeozeroGeometry for Gpx<'a> {
    fn process_geom<P: crate::GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        read_gpx(&mut self.0.as_bytes(), processor)
    }
}

pub fn read_gpx<R: io::Read, P: crate::GeomProcessor>(
    reader: &mut R,
    processor: &mut P,
) -> crate::error::Result<()> {
    let gpx_reader = match gpx::read(reader) {
        Ok(r) => r,
        Err(e) => return Err(GeozeroError::Geometry(e.to_string())),
    };

    process_top_level_waypoints(&gpx_reader, processor)?;
    process_top_level_tracks(&gpx_reader, processor)?;
    process_top_level_routes(&gpx_reader, processor)?;

    Ok(())
}

fn process_top_level_waypoints<P: crate::GeomProcessor>(
    gpx_reader: &gpx::Gpx,
    processor: &mut P,
) -> crate::error::Result<()> {
    if gpx_reader.waypoints.is_empty() {
        return Ok(());
    }
    processor.multipoint_begin(gpx_reader.waypoints.len(), 0)?;
    process_waypoints_iter(gpx_reader.waypoints.iter(), processor)?;
    processor.multipoint_end(0)?;
    Ok(())
}

fn process_top_level_tracks<P: crate::GeomProcessor>(
    gpx_reader: &gpx::Gpx,
    processor: &mut P,
) -> crate::error::Result<()> {
    if gpx_reader.tracks.is_empty() {
        return Ok(());
    }
    for (index, track) in gpx_reader.tracks.iter().enumerate() {
        process_track_segments(track, processor, index)?;
    }
    Ok(())
}

fn process_track_segments<P: crate::GeomProcessor>(
    track: &gpx::Track,
    processor: &mut P,
    index: usize,
) -> crate::error::Result<()> {
    if track.segments.is_empty() {
        return Ok(());
    }
    processor.multilinestring_begin(track.segments.len(), index)?;
    for (index, segment) in track.segments.iter().enumerate() {
        process_track_segment(segment, processor, index)?;
    }
    processor.multilinestring_end(index)?;
    Ok(())
}

fn process_track_segment<P: crate::GeomProcessor>(
    segment: &gpx::TrackSegment,
    processor: &mut P,
    index: usize,
) -> crate::error::Result<()> {
    if segment.points.is_empty() {
        return Ok(());
    }
    processor.linestring_begin(false, segment.points.len(), index)?;
    process_waypoints_iter(segment.points.iter(), processor)?;
    processor.linestring_end(false, index)?;
    Ok(())
}

fn process_top_level_routes<P: crate::GeomProcessor>(
    gpx_reader: &gpx::Gpx,
    processor: &mut P,
) -> crate::error::Result<()> {
    if gpx_reader.routes.is_empty() {
        return Ok(());
    }
    processor.multilinestring_begin(gpx_reader.routes.len(), 0)?;
    for (index, route) in gpx_reader.routes.iter().enumerate() {
        process_route(route, processor, index)?;
    }
    processor.multilinestring_end(0)?;
    Ok(())
}

fn process_route<P: crate::GeomProcessor>(
    route: &gpx::Route,
    processor: &mut P,
    index: usize,
) -> crate::error::Result<()> {
    if route.points.is_empty() {
        return Ok(());
    }
    processor.linestring_begin(false, route.points.len(), index)?;
    process_waypoints_iter(route.points.iter(), processor)?;
    processor.linestring_end(false, index)?;
    Ok(())
}

fn process_waypoints_iter<'a, P: crate::GeomProcessor>(
    iter: impl Iterator<Item = &'a gpx::Waypoint>,
    processor: &mut P,
) -> crate::error::Result<()> {
    for (index, waypoint) in iter.enumerate() {
        let point = waypoint.point();
        processor.xy(point.x(), point.y(), index)?;
    }
    Ok(())
}
