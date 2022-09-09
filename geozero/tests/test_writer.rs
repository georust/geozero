use geozero::error::Result;

#[derive(Debug, PartialEq)]
pub enum Cmd {
    Xy {
        idx: usize,
        x: f64,
        y: f64,
    },
    Coordinate {
        idx: usize,
        x: f64,
        y: f64,
        z: Option<f64>,
        m: Option<f64>,
        t: Option<f64>,
        tm: Option<u64>,
    },
    PointBegin {
        idx: usize,
    },
    PointEnd {
        idx: usize,
    },
    EmptyPoint {
        idx: usize,
    },
    MultiPointBegin {
        idx: usize,
    },
    MultiPointEnd {
        idx: usize,
    },
    LineStringBegin {
        idx: usize,
    },
    LineStringEnd {
        idx: usize,
    },
    MultiLineStringBegin {
        idx: usize,
    },
    MultiLineStringEnd {
        idx: usize,
    },
    PolygonBegin {
        idx: usize,
    },
    PolygonEnd {
        idx: usize,
    },
    MultiPolygonBegin {
        idx: usize,
    },
    MultiPolygonEnd {
        idx: usize,
    },
    GeometryCollectionBegin {
        idx: usize,
    },
    GeometryCollectionEnd {
        idx: usize,
    },
}

#[derive(Default)]
pub struct TestWriter(pub Vec<Cmd>);

impl geozero::GeomProcessor for TestWriter {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        self.0.push(Cmd::Xy { idx, x, y });
        Ok(())
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
        self.0.push(Cmd::Coordinate { idx, x, y, z, m, t, tm });
        Ok(())
    }

    fn point_begin(&mut self, idx: usize) -> Result<()> {
        self.0.push(Cmd::PointBegin { idx });
        Ok(())
    }
    fn point_end(&mut self, idx: usize) -> Result<()> {
        self.0.push(Cmd::PointEnd { idx });
        Ok(())
    }

    fn empty_point(&mut self, idx: usize) -> Result<()> {
        self.0.push(Cmd::EmptyPoint { idx });
        Ok(())
    }

    fn multipoint_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.0.push(Cmd::MultiPointBegin { idx });
        Ok(())
    }

    fn multipoint_end(&mut self, idx: usize) -> Result<()> {
        self.0.push(Cmd::MultiPointEnd { idx });
        Ok(())
    }

    fn linestring_begin(&mut self, _tagged: bool, _size: usize, idx: usize) -> Result<()> {
        self.0.push(Cmd::LineStringBegin { idx });
        Ok(())
    }

    fn linestring_end(&mut self, _tagged: bool, idx: usize) -> Result<()> {
        self.0.push(Cmd::LineStringEnd { idx });
        Ok(())
    }

    fn multilinestring_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.0.push(Cmd::MultiLineStringBegin { idx });
        Ok(())
    }

    fn multilinestring_end(&mut self, idx: usize) -> Result<()> {
        self.0.push(Cmd::MultiLineStringEnd { idx });
        Ok(())
    }

    fn polygon_begin(&mut self, _tagged: bool, _size: usize, idx: usize) -> Result<()> {
        self.0.push(Cmd::PolygonBegin { idx });
        Ok(())
    }

    fn polygon_end(&mut self, _tagged: bool, idx: usize) -> Result<()> {
        self.0.push(Cmd::PolygonEnd { idx });
        Ok(())
    }

    fn multipolygon_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.0.push(Cmd::MultiPolygonBegin { idx });
        Ok(())
    }

    fn multipolygon_end(&mut self, idx: usize) -> Result<()> {
        self.0.push(Cmd::MultiPolygonEnd { idx });
        Ok(())
    }

    fn geometrycollection_begin(&mut self, _size: usize, idx: usize) -> Result<()> {
        self.0.push(Cmd::GeometryCollectionBegin { idx });
        Ok(())
    }

    fn geometrycollection_end(&mut self, idx: usize) -> Result<()> {
        self.0.push(Cmd::GeometryCollectionEnd { idx });
        Ok(())
    }

    fn circularstring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn circularstring_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn compoundcurve_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn compoundcurve_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn curvepolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn curvepolygon_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn multicurve_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn multicurve_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn multisurface_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn multisurface_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn triangle_begin(&mut self, _tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn triangle_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn polyhedralsurface_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn polyhedralsurface_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn tin_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        Ok(())
    }

    fn tin_end(&mut self, _idx: usize) -> Result<()> {
        Ok(())
    }
}
