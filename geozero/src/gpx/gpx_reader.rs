use crate::error::GeozeroError;
use std::io;

/// GPX geometry collection
pub struct Gpx<'a>(pub &'a str);

impl<'a> crate::GeozeroGeometry for Gpx<'a> {
    fn process_geom<P: crate::GeomProcessor>(&self, processor: &mut P) -> crate::error::Result<()> {
        read_gpx(&mut self.0.as_bytes(), processor)
    }
}

/// GPX reader
pub struct GpxReader<R: io::Read>(pub R);

impl<R: io::Read> crate::GeozeroDatasource for GpxReader<R> {
    fn process<P: crate::FeatureProcessor>(
        &mut self,
        processor: &mut P,
    ) -> crate::error::Result<()> {
        read_gpx(&mut self.0, processor)
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

    let mut index = 0;
    let size = gpx_reader.waypoints.len() + gpx_reader.tracks.len() + gpx_reader.routes.len();

    processor.geometrycollection_begin(size, 0)?;
    process_top_level_waypoints(&gpx_reader, processor, &mut index)?;
    process_top_level_tracks(&gpx_reader, processor, &mut index)?;
    process_top_level_routes(&gpx_reader, processor, &mut index)?;
    processor.geometrycollection_end(0)
}

fn process_top_level_waypoints<P: crate::GeomProcessor>(
    gpx_reader: &gpx::Gpx,
    processor: &mut P,
    index: &mut usize,
) -> crate::error::Result<()> {
    if gpx_reader.waypoints.is_empty() {
        return Ok(());
    }
    process_waypoints_iter(gpx_reader.waypoints.iter(), processor, index, true)?;
    Ok(())
}

fn process_top_level_tracks<P: crate::GeomProcessor>(
    gpx_reader: &gpx::Gpx,
    processor: &mut P,
    index: &mut usize,
) -> crate::error::Result<()> {
    for track in &gpx_reader.tracks {
        process_track_segments(track, processor, *index)?;
        *index += 1;
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
    for (inner_index, segment) in track.segments.iter().enumerate() {
        process_track_segment(segment, processor, inner_index)?;
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
    process_waypoints_iter(segment.points.iter(), processor, &mut 0, false)?;
    processor.linestring_end(false, index)?;
    Ok(())
}

fn process_top_level_routes<P: crate::GeomProcessor>(
    gpx_reader: &gpx::Gpx,
    processor: &mut P,
    index: &mut usize,
) -> crate::error::Result<()> {
    if gpx_reader.routes.is_empty() {
        return Ok(());
    }
    processor.multilinestring_begin(gpx_reader.routes.len(), *index)?;
    for (inner_index, route) in gpx_reader.routes.iter().enumerate() {
        process_route(route, processor, inner_index)?;
    }
    processor.multilinestring_end(*index)?;
    *index += 1;
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
    process_waypoints_iter(route.points.iter(), processor, &mut 0, false)?;
    processor.linestring_end(false, index)
}

fn process_waypoints_iter<'a, P: crate::GeomProcessor>(
    iter: impl Iterator<Item = &'a gpx::Waypoint>,
    processor: &mut P,
    index: &mut usize,
    wrap_point: bool,
) -> crate::error::Result<()> {
    for waypoint in iter {
        let point = waypoint.point();
        if wrap_point {
            processor.point_begin(*index)?;
            processor.xy(point.x(), point.y(), 0)?;
            processor.point_end(*index)?;
        } else {
            processor.xy(point.x(), point.y(), *index)?;
        }
        *index += 1;
    }
    Ok(())
}
