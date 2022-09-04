use crate::error::{GeozeroError, Result};
use std::io;

pub fn read_gpx<R: io::Read, P: crate::GeomProcessor>(
    reader: &mut R,
    processor: &mut P,
) -> Result<()> {
    let gpx_reader = gpx::read(reader).unwrap(); // TODO;
    let mut index = 0; // WHAT IS THIS FOR

    // Waypoints
    processor.multipoint_begin(gpx_reader.waypoints.len(), index)?;
    for waypoint in &gpx_reader.waypoints {
        let point = waypoint.point();
        processor.xy(point.x(), point.y(), index)?;
    }
    processor.multipoint_end(index)?;

    // Tracks
    processor.multilinestring_begin(todo!(), index)?; // TODO: what is the right size
    for track in &gpx_reader.tracks {
        // TODO: is this multi<multi<linestring>>?
        for segment in track.segments.iter() {
            processor.linestring_begin(false, todo!(), index)?; // TODO: what is the right size
            for waypoint in segment.points {
                let point = waypoint.point();
                processor.xy(point.x(), point.y(), index)?;
            }
            processor.linestring_end(false, index)?;
        }
    }
    processor.multilinestring_end(index)?;

    // Routes
    todo!()
}

fn process_point() {

}
