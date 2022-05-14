use crate::error::Result;
use crate::{
    ColumnValue, CoordDimensions, FeatureProcessor, GeomProcessor, GeozeroDatasource,
    PropertyProcessor,
};

use delegate::delegate;
use proj::Proj;

use std::rc::Rc;

/// ```
/// use proj::Proj;
/// use geozero::{ToProjected, ProcessToJson, wkt::WktStr};
///
/// // Convert from [NAD 83 US Survey Feet](https://epsg.io/2230) to [NAD 83 Meters](https://epsg.io/26946) Using EPSG Codes
/// let from = "EPSG:2230";
/// let to = "EPSG:26946";
/// let proj = Proj::new_known_crs(from, to, None).unwrap();
///
/// let wkt = WktStr("POINT (4760096.421921 3744293.729449)");
/// let expected = r#"{"type": "Point", "coordinates": [1450880.2910605022,1141263.0111604782]}"#;
/// assert_eq!(expected, &wkt.to_projected(proj).to_json().unwrap());
/// ```
pub trait ToProjected: GeozeroDatasource + Sized {
    fn to_projected(self, proj: Proj) -> ProjWrappedDataSource<Self>;
}

impl<Input: GeozeroDatasource> ToProjected for Input {
    fn to_projected(self, proj: Proj) -> ProjWrappedDataSource<Input> {
        ProjWrappedDataSource {
            proj: Rc::new(proj),
            input: self,
        }
    }
}

pub struct ProjWrappedDataSource<Input: GeozeroDatasource> {
    proj: Rc<Proj>,
    input: Input,
}

struct ProjPreprocessor<'a> {
    proj: Rc<Proj>,
    output: &'a mut dyn FeatureProcessor,
}

impl<Input: GeozeroDatasource> GeozeroDatasource for ProjWrappedDataSource<Input> {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        let mut wrapped = ProjPreprocessor {
            proj: self.proj.clone(),
            output: processor,
        };
        self.input.process(&mut wrapped)
    }
}

impl FeatureProcessor for ProjPreprocessor<'_> {
    delegate! {
         to self.output {
             fn dataset_begin(&mut self, name: Option<&str>) -> Result<()>;
             fn dataset_end(&mut self) -> Result<()>;
             fn feature_begin(&mut self, idx: u64) -> Result<()>;
             fn feature_end(&mut self, idx: u64) -> Result<()>;
             fn properties_begin(&mut self) -> Result<()>;
             fn properties_end(&mut self) -> Result<()>;
             fn geometry_begin(&mut self) -> Result<()>;
             fn geometry_end(&mut self) -> Result<()>;
        }
    }
}

impl PropertyProcessor for ProjPreprocessor<'_> {
    delegate! {
        to self.output {
            fn property(&mut self, idx: usize, name: &str, value: &ColumnValue) -> Result<bool>;
        }
    }
}

impl GeomProcessor for ProjPreprocessor<'_> {
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
        let (x, y) = self.proj.convert((x, y)).unwrap();
        self.output.coordinate(x, y, z, m, t, tm, idx)
    }

    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        let (x, y) = self.proj.convert((x, y)).unwrap();
        self.output.xy(x, y, idx)
    }

    delegate! {
        to self.output {
            fn dimensions(&self) -> CoordDimensions;
            fn multi_dim(&self) -> bool;
            fn srid(&mut self, srid: Option<i32>) -> Result<()>;
            fn empty_point(&mut self, idx: usize) -> Result<()>;
            fn point_begin(&mut self, idx: usize) -> Result<()>;
            fn point_end(&mut self, idx: usize) -> Result<()>;
            fn multipoint_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn multipoint_end(&mut self, idx: usize) -> Result<()>;
            fn linestring_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()>;
            fn linestring_end(&mut self, tagged: bool, idx: usize) -> Result<()>;
            fn multilinestring_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn multilinestring_end(&mut self, idx: usize) -> Result<()>;
            fn polygon_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()>;
            fn polygon_end(&mut self, tagged: bool, idx: usize) -> Result<()>;
            fn multipolygon_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn multipolygon_end(&mut self, idx: usize) -> Result<()>;
            fn geometrycollection_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn geometrycollection_end(&mut self, idx: usize) -> Result<()>;
            fn circularstring_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn circularstring_end(&mut self, idx: usize) -> Result<()>;
            fn compoundcurve_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn compoundcurve_end(&mut self, idx: usize) -> Result<()>;
            fn curvepolygon_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn curvepolygon_end(&mut self, idx: usize) -> Result<()>;
            fn multicurve_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn multicurve_end(&mut self, idx: usize) -> Result<()>;
            fn multisurface_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn multisurface_end(&mut self, idx: usize) -> Result<()>;
            fn triangle_begin(&mut self, tagged: bool, size: usize, idx: usize) -> Result<()>;
            fn triangle_end(&mut self, tagged: bool, idx: usize) -> Result<()>;
            fn polyhedralsurface_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn polyhedralsurface_end(&mut self, idx: usize) -> Result<()>;
            fn tin_begin(&mut self, size: usize, idx: usize) -> Result<()>;
            fn tin_end(&mut self, idx: usize) -> Result<()>;
        }
    }
}
