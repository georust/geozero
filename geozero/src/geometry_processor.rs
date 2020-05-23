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
