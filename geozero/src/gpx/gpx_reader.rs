use std::io;

pub fn read_gpx<R: io::Read, P: crate::GeomProcessor>(
    reader: &mut R,
    processor: &mut P,
) -> crate::error::Result<()> {
    let gpx_reader = gpx::read(reader).unwrap(); // TODO;

    // Waypoints
    processor.multipoint_begin(gpx_reader.waypoints.len(), 0)?;
    for waypoint in &gpx_reader.waypoints {
        let point = waypoint.point();
        processor.xy(point.x(), point.y(), 0)?;
    }
    processor.multipoint_end(0)?;

    // Tracks
    for (index, track) in gpx_reader.tracks.iter().enumerate() {
        processor.multilinestring_begin(track.segments.len(), index)?;
        for segment in track.segments.iter() {
            processor.linestring_begin(false, segment.points.len(), index)?;
            for waypoint in &segment.points {
                let point = waypoint.point();
                processor.xy(point.x(), point.y(), index)?;
            }
            processor.linestring_end(false, index)?;
        }
        processor.multilinestring_end(index)?;
    }

    // Routes
    processor.multilinestring_begin(gpx_reader.routes.len(), 0)?;
    for (index, route) in gpx_reader.routes.iter().enumerate() {
        processor.linestring_begin(false, route.points.len(), index)?;
        for waypoint in route.points.iter() {
            let point = waypoint.point();
            processor.xy(point.x(), point.y(), index)?;
        }
        processor.linestring_end(false, index)?;
    }
    processor.multilinestring_end(0)?;

    Ok(())
}

#[cfg(test)]
mod test {
    #[test]
    fn test_basic() {

    }
}
