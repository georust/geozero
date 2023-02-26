use crate::error::{GeozeroError, Result};
use crate::{FeatureProcessor, GeomProcessor, PropertyProcessor};
use geo_types::*;
use std::mem;

/// Generator for geo-types geometry type.
pub struct GeoWriter {
    geoms: Vec<Geometry<f64>>,
    // Stack of any in-progress (potentially nested) GeometryCollections
    collections: Vec<Vec<Geometry<f64>>>,
    // In-progress multi-polygon
    polygons: Option<Vec<Polygon<f64>>>,
    // In-progress polygon or multi_linestring
    line_strings: Option<Vec<LineString<f64>>>,
    // In-progress point or line_string
    coords: Option<Vec<Coordinate<f64>>>,
}

impl GeoWriter {
    pub fn new() -> GeoWriter {
        GeoWriter {
            geoms: Vec::new(),
            coords: None,
            line_strings: None,
            polygons: None,
            collections: Vec::new(),
        }
    }

    pub fn take_geometry(&mut self) -> Option<Geometry<f64>> {
        match self.geoms.len() {
            0 => None,
            1 => Some(self.geoms.pop().unwrap()),
            _ => {
                let geoms = std::mem::take(&mut self.geoms);
                Some(Geometry::GeometryCollection(GeometryCollection(geoms)))
            }
        }
    }

    fn finish_geometry(&mut self, geometry: Geometry<f64>) -> Result<()> {
        // Add the geometry to a collection if we're in the middle of processing
        // a (potentially nested) collection
        if let Some(most_recent_collection) = self.collections.last_mut() {
            most_recent_collection.push(geometry);
        } else {
            self.geoms.push(geometry);
        }
        Ok(())
    }
}

impl GeomProcessor for GeoWriter {
    fn xy(&mut self, x: f64, y: f64, _idx: usize) -> Result<()> {
        let coords = self
            .coords
            .as_mut()
            .ok_or(GeozeroError::Geometry("Not ready for coords".to_string()))?;
        coords.push(coord!(x: x, y: y));
        Ok(())
    }

    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        debug_assert!(self.coords.is_none());
        self.coords = Some(Vec::with_capacity(1));
        Ok(())
    }

    fn point_end(&mut self, _idx: usize) -> Result<()> {
        let coords = self
            .coords
            .take()
            .ok_or(GeozeroError::Geometry("No coords for Point".to_string()))?;
        debug_assert!(coords.len() == 1);
        self.finish_geometry(Point(coords[0]).into())
    }

    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.coords.is_none());
        self.coords = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn multipoint_end(&mut self, _idx: usize) -> Result<()> {
        let coords = self.coords.take().ok_or(GeozeroError::Geometry(
            "No coords for MultiPoint".to_string(),
        ))?;
        let points: Vec<Point<_>> = coords.into_iter().map(From::from).collect();
        self.finish_geometry(MultiPoint(points).into())
    }

    fn linestring_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.coords.is_none());
        self.coords = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn linestring_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        let coords = self.coords.take().ok_or(GeozeroError::Geometry(
            "No coords for LineString".to_string(),
        ))?;
        let line_string = LineString(coords);
        if tagged {
            self.finish_geometry(line_string.into())?;
        } else {
            let line_strings = self.line_strings.as_mut().ok_or(GeozeroError::Geometry(
                "Missing container for LineString".to_string(),
            ))?;
            line_strings.push(line_string);
        }
        Ok(())
    }

    fn multilinestring_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.line_strings.is_none());
        self.line_strings = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn multilinestring_end(&mut self, _idx: usize) -> Result<()> {
        let line_strings = self.line_strings.take().ok_or(GeozeroError::Geometry(
            "No LineStrings for MultiLineString".to_string(),
        ))?;
        self.finish_geometry(MultiLineString(line_strings).into())
    }

    fn polygon_begin(&mut self, _tagged: bool, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.line_strings.is_none());
        self.line_strings = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn polygon_end(&mut self, tagged: bool, _idx: usize) -> Result<()> {
        let mut line_strings = self.line_strings.take().ok_or(GeozeroError::Geometry(
            "Missing LineStrings for Polygon".to_string(),
        ))?;

        let polygon = if line_strings.is_empty() {
            Polygon::new(LineString(vec![]), vec![])
        } else {
            let exterior = line_strings.remove(0);
            Polygon::new(exterior, mem::take(&mut line_strings))
        };

        if tagged {
            self.finish_geometry(polygon.into())?;
        } else {
            let polygons = self.polygons.as_mut().ok_or(GeozeroError::Geometry(
                "Missing container for Polygon".to_string(),
            ))?;
            polygons.push(polygon);
        }
        Ok(())
    }

    fn multipolygon_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        debug_assert!(self.polygons.is_none());
        self.polygons = Some(Vec::with_capacity(size));
        Ok(())
    }

    fn multipolygon_end(&mut self, _idx: usize) -> Result<()> {
        let polygons = self.polygons.take().ok_or(GeozeroError::Geometry(
            "Missing polygons for MultiPolygon".to_string(),
        ))?;
        self.finish_geometry(MultiPolygon(polygons).into())
    }

    fn geometrycollection_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.collections.push(Vec::with_capacity(size));
        Ok(())
    }

    fn geometrycollection_end(&mut self, _idx: usize) -> Result<()> {
        let geometries = self.collections.pop().ok_or(GeozeroError::Geometry(
            "Unexpected geometry type".to_string(),
        ))?;

        self.finish_geometry(Geometry::GeometryCollection(GeometryCollection(geometries)))
    }
}

impl PropertyProcessor for GeoWriter {}

impl FeatureProcessor for GeoWriter {}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::{read_geojson, GeoJson};
    use crate::ToGeo;
    use geo::algorithm::coords_iter::CoordsIter;

    #[test]
    fn line_string() -> Result<()> {
        let geojson = r#"{"type": "LineString", "coordinates": [[1875038.447610231,-3269648.6879248763],[1874359.641504197,-3270196.812984864],[1874141.0428635243,-3270953.7840121365],[1874440.1778162003,-3271619.4315206874],[1876396.0598222911,-3274138.747656357],[1876442.0805243007,-3275052.60551469],[1874739.312657555,-3275457.333765534]]}"#;
        let mut geo = GeoWriter::new();
        assert!(read_geojson(geojson.as_bytes(), &mut geo).is_ok());
        let geom = geo.take_geometry().unwrap();
        match geom {
            Geometry::LineString(line) => {
                assert_eq!(line.coords_count(), 7);
                assert_eq!(
                    line.points().next().unwrap(),
                    Point::new(1875038.447610231, -3269648.6879248763)
                );
            }
            _ => assert!(false),
        }
        Ok(())
    }

    #[test]
    fn multipolygon() -> Result<()> {
        let geojson = GeoJson(
            r#"{"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}"#,
        );
        let geo = geojson.to_geo().unwrap();
        println!("{:?}", geo);
        match geo {
            Geometry::MultiPolygon(mp) => {
                let poly = mp.into_iter().next().unwrap();
                assert_eq!(
                    poly.exterior().points().next().unwrap(),
                    Point::new(173.020375, -40.919052)
                );
            }
            _ => assert!(false),
        }
        Ok(())
    }

    #[test]
    fn geometry_collection() {
        use crate::wkt::WktStr;
        let wkt = WktStr("GEOMETRYCOLLECTION(POINT(1 2),LINESTRING(1 2,3 4))");
        let actual = wkt.to_geo().unwrap();

        use geo_types::{line_string, point, Geometry, GeometryCollection};

        let expected = Geometry::GeometryCollection(GeometryCollection(vec![
            point!(x: 1.0, y: 2.0).into(),
            line_string![(x: 1.0, y: 2.0), (x: 3.0, y: 4.0)].into(),
        ]));

        assert_eq!(expected, actual);
    }

    #[test]
    fn nested_geometry_collections() {
        use crate::wkt::WktStr;
        let wkt = WktStr("GEOMETRYCOLLECTION(POINT(1 2),GEOMETRYCOLLECTION(LINESTRING(1 2,3 4), MULTIPOINT(1 2, 3 4, 5 6)),POINT(7 8))");
        let actual = wkt.to_geo().unwrap();

        use geo_types::{line_string, point, Geometry, GeometryCollection, MultiPoint};

        let expected = Geometry::GeometryCollection(GeometryCollection(vec![
            point!(x: 1.0, y: 2.0).into(),
            Geometry::GeometryCollection(GeometryCollection(vec![
                line_string![(x: 1.0, y: 2.0), (x: 3.0, y: 4.0)].into(),
                MultiPoint(vec![
                    point!(x: 1.0, y: 2.0),
                    point!(x: 3.0, y: 4.0),
                    point!(x: 5.0, y: 6.0),
                ])
                .into(),
            ])),
            point!(x: 7.0, y: 8.0).into(),
        ]));

        assert_eq!(expected, actual);
    }

    #[test]
    fn complex() {
        use crate::wkt::WktStr;
        let wkt = WktStr("GEOMETRYCOLLECTION (LINESTRING (6308869.40378 356821.22669, 6308867.893 356822.41744, 6308852.75314 356830.22159, 6308869.92754 356844.26638), LINESTRING (6308755.07971 356674.51686, 6308784.81355 356719.16757, 6308815.20022 356765.46178, 6308829.63774 356763.22832, 6308852.87023 356759.82402, 6308867.19982 356771.06823, 6308875.40631 356796.20162, 6308872.51907 356815.17242), LINESTRING (6308874.12086 356813.73392, 6308876.83028 356795.77697, 6308868.23871 356770.06254, 6308853.09618 356758.29456, 6308815.86529 356763.89689, 6308799.76731 356739.37835, 6308747.77971 356662.11613, 6308746.55411 356661.61702, 6308744.06545 356657.72563, 6308731.77184 356668.45076, 6308699.45221 356683.15463, 6308682.44689 356684.63193, 6308654.96629 356683.66846, 6308636.13879 356680.0482, 6308618.19888 356671.76352, 6308608.41685 356661.79428, 6308578.7973 356592.35062, 6308545.33908 356542.14886, 6308517.52088 356509.38474, 6308505.40266 356506.84141, 6308493.59689 356506.98067, 6308375.07918 356520.46209), LINESTRING (6308877.92941 356819.50984, 6309072.26249 356514.14689, 6309073.44938 356513.3739, 6309076.25423 356511.31751, 6309096.05004 356528.52014, 6309103.33938 356535.32615, 6309107.49584 356539.20699, 6309107.78601 356539.47793, 6309119.09139 356550.03322, 6309137.04465 356567.13752, 6309137.6323 356567.69515, 6309138.92096 356568.91355, 6309138.46355 356569.69798, 6309150.68532 356566.34027, 6309151.94333 356567.03108, 6309157.81557 356565.41779, 6309161.54152 356564.33408, 6309174.6464 356579.77423, 6309175.71622 356581.0361, 6309177.25892 356582.84545, 6309225.37695 356611.76515, 6309226.90588 356612.65173, 6309229.72021 356614.34101, 6309232.64678 356598.75445, 6309244.10246 356528.49893, 6309251.20809 356487.90256, 6309252.35489 356481.34967, 6309258.41778 356442.34047, 6309258.56036 356441.19511, 6309258.76115 356440.13123, 6309260.99127 356426.22389, 6309258.49745 356425.57244, 6309240.94882 356422.48836, 6309240.53276 356422.37171, 6309240.10958 356422.29068), LINESTRING (6308870.96141 356823.05522, 6308881.43519 356846.04558, 6308859.94336 356857.75024, 6308859.6305 356857.95378, 6308893.96675 356932.14467, 6308921.19517 356993.60222, 6308942.68768 357040.82051, 6308961.42173 357079.52481, 6308976.48471 357108.08898, 6308992.14194 357136.52543, 6309018.60922 357184.68892, 6309024.87557 357193.57884, 6309025.31785 357194.20629, 6309028.73486 357199.05392, 6309045.86114 357220.97586, 6309078.85225 357261.01696, 6309131.17986 357323.22098, 6309184.03434 357388.33409, 6309212.61182 357423.54026, 6309252.80543 357467.20429, 6309288.51836 357504.59499, 6309318.98068 357536.37443, 6309366.01084 357588.07961, 6309383.32941 357609.89089, 6309383.33718 357609.92579, 6309383.36584 357611.49516), POLYGON ((6309096.87876754 357058.96992573235, 6309100.9240038069 357067.89795246266, 6309103.1497403858 357077.44361610821, 6309103.4704434676 357087.24008216924, 6309101.8737886148 357096.91087794991, 6309098.421134375 357106.08436019259, 6309093.2451643161 357114.40799712023, 6309086.5447880644 357121.56191603432, 6309078.5774973193 357127.27119584358, 6309013.594489282 357164.78915304772, 6309004.6664625369 357168.8343893392, 6308995.1207988719 357171.06012593472, 6308985.3243327877 357171.38082902494, 6308975.6535369828 357169.78417417, 6308966.4800547194 357166.33151992067, 6308958.156417775 357161.1555498438, 6308951.0024988521 357154.45517356717, 6308945.293219042 357146.48788279406, 6308795.0043396624 356886.1799069175, 6308790.959103398 356877.251880196, 6308788.7333668182 356867.70621655986, 6308788.4126637317 356857.90975050797, 6308790.0093185771 356848.2389547351, 6308793.4619728047 356839.0654724979, 6308798.6379428506 356830.74183557258, 6308805.3383190883 356823.587916657, 6308813.3056098176 356817.87863684172, 6308878.2886178084 356780.36067953659, 6308887.2166445563 356776.315443229, 6308896.7623082288 356774.08970661991, 6308906.5587743223 356773.76900351944, 6308916.2295701364 356775.36565836804, 6308925.403052411 356778.81831261492, 6308933.7266893657 356783.99428269314, 6308940.8806082979 356790.69465897442, 6308946.5898881136 356798.66194975481, 6309096.87876754 357058.96992573235)))");

        assert!(wkt.to_geo().is_ok());
    }

    #[test]
    fn to_geo() -> Result<()> {
        let geom: geo_types::Geometry<f64> = geo_types::Point::new(10.0, 20.0).into();
        assert_eq!(geom.clone().to_geo().unwrap(), geom);
        Ok(())
    }
}
