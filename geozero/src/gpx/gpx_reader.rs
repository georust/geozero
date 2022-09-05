use crate::error::GeozeroError;
use std::io;

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
    process_waypoints_iter(gpx_reader.waypoints.iter(), processor, 0)?;
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
    for segment in &track.segments {
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
    process_waypoints_iter(segment.points.iter(), processor, index)?;
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
    process_waypoints_iter(route.points.iter(), processor, index)?;
    processor.linestring_end(false, index)?;
    Ok(())
}

fn process_waypoints_iter<'a, P: crate::GeomProcessor>(
    iter: impl Iterator<Item = &'a gpx::Waypoint>,
    processor: &mut P,
    index: usize,
) -> crate::error::Result<()> {
    for waypoint in iter {
        let point = waypoint.point();
        processor.xy(point.x(), point.y(), index)?;
    }
    Ok(())
}

#[cfg(test)]
#[cfg(feature = "with-geo")]
mod test {
    use super::*;
    use geo_types::geometry::*;
    use std::io;

    #[test]
    fn test_empty_str() {
        let gpx_str = "";
        let mut cursor = io::Cursor::new(gpx_str);
        let mut geo_writer = crate::geo_types::GeoWriter::new();
        assert!(matches!(
            read_gpx(&mut cursor, &mut geo_writer),
            Err(GeozeroError::Geometry(_)),
        ));
    }

    #[test]
    fn test_wikipedia_example() {
        let gpx_str = include_str!("fixtures/wikipedia_example.gpx");
        let mut cursor = io::Cursor::new(gpx_str);
        let mut geo_writer = crate::geo_types::GeoWriter::new();
        read_gpx(&mut cursor, &mut geo_writer).unwrap();
        let actual = geo_writer.take_geometry().unwrap();
        let expected = Geometry::MultiLineString(MultiLineString(vec![LineString(vec![
            Coordinate {
                x: -122.326897,
                y: 47.644548,
            },
            Coordinate {
                x: -122.326897,
                y: 47.644548,
            },
            Coordinate {
                x: -122.326897,
                y: 47.644548,
            },
        ])]));
        assert_eq!(expected, actual);
    }
}
