//! geo-types conversions.
pub(crate) mod geo_types_reader;
pub(crate) mod geo_types_writer;

pub use geo_types_reader::*;
pub use geo_types_writer::*;

pub(crate) mod conversion {
    use super::geo_types_writer::*;
    use crate::error::{GeozeroError, Result};
    use crate::events::GeomVisitor;
    use crate::GeozeroGeometry;

    /// Convert to geo-types Geometry.
    pub trait ToGeo {
        /// Convert to geo-types Geometry.
        fn to_geo(&self) -> Result<geo_types::Geometry<f64>>;
    }

    impl<T: GeozeroGeometry> ToGeo for T {
        fn to_geo(&self) -> Result<geo_types::Geometry<f64>> {
            let mut writer = GeoWriter::new();
            let mut visitor = GeomVisitor::new(&mut writer);
            self.process_geom(&mut visitor)?;
            writer
                .take_geometry()
                .ok_or(GeozeroError::Geometry("Missing Geometry".to_string()))
        }
    }
}

#[cfg(feature = "with-wkb")]
mod wkb {
    use super::geo_types_writer::*;
    use crate::error::{GeozeroError, Result};
    use crate::events::GeomVisitor;
    use crate::wkb::{FromWkb, WkbDialect};
    use std::io::Read;

    impl FromWkb for geo_types::Geometry<f64> {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut writer = GeoWriter::new();
            let mut visitor = GeomVisitor::new(&mut writer);
            crate::wkb::process_wkb_type_geom(rdr, &mut visitor, dialect)?;
            writer
                .take_geometry()
                .ok_or(GeozeroError::Geometry("Missing Geometry".to_string()))
        }
    }
}

#[cfg(test)]
mod test {
    use crate::geo_types::conversion::ToGeo;
    use crate::geojson::GeoJsonString;

    use geo_types::{Geometry, GeometryCollection, Point};
    use serde_json::json;

    #[test]
    fn from_geojson_feature_collection_of_points() {
        let geojson = GeoJsonString(
            json!({
                "type": "FeatureCollection",
                "features": [
                    {
                        "type": "Feature",
                        "properties": {
                            "population": 100
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [10.0, 45.0]
                        }
                    },
                    {
                        "type": "Feature",
                        "properties": {
                            "population": 200
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [20.0, 45.0]
                        }
                    }
                ]
            })
            .to_string(),
        );

        let actual = geojson.to_geo().unwrap();
        let expected = Geometry::GeometryCollection(GeometryCollection::<f64>(vec![
            Point::new(10.0, 45.0).into(),
            Point::new(20.0, 45.0).into(),
        ]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn from_geojson_feature_collection_geometry_collection_and_point() {
        let geojson = GeoJsonString(
            json!({
                "type": "FeatureCollection",
                "features": [
                    {
                        "type": "Feature",
                        "properties": {
                            "population": 100
                        },
                        "geometry": {
                            "type": "GeometryCollection",
                            "geometries": [
                                {
                                    "type": "Point",
                                    "coordinates": [10.1, 45.0]
                                },
                                {
                                    "type": "Point",
                                    "coordinates": [10.2, 45.0]
                                }
                            ]
                        }
                    },
                    {
                        "type": "Feature",
                        "properties": {
                            "population": 200
                        },
                        "geometry": {
                            "type": "Point",
                            "coordinates": [20.0, 45.0]
                        }
                    }
                ]
            })
            .to_string(),
        );

        let actual = geojson.to_geo().unwrap();
        let expected = Geometry::GeometryCollection(GeometryCollection::<f64>(vec![
            Geometry::GeometryCollection(GeometryCollection::<f64>(vec![
                Point::new(10.1, 45.0).into(),
                Point::new(10.2, 45.0).into(),
            ])),
            Point::new(20.0, 45.0).into(),
        ]));
        assert_eq!(expected, actual);
    }

    #[test]
    fn from_geojson_point_feature() {
        let geojson = GeoJsonString(
            json!({
                "type": "Feature",
                "properties": {
                    "population": 100
                },
                "geometry": {
                    "type": "Point",
                    "coordinates": [10.0, 45.0]
                }
            })
            .to_string(),
        );

        use geo_types::{Geometry, Point};

        let actual = geojson.to_geo().unwrap();
        let expected = Geometry::Point(Point::new(10.0, 45.0));
        assert_eq!(expected, actual);
    }

    #[test]
    fn from_geojson_geometry_collection_feature() {
        let geojson = GeoJsonString(
            json!({
                "type": "Feature",
                "properties": {
                    "population": 100
                },
                "geometry": {
                    "type": "GeometryCollection",
                    "geometries": [
                        {
                            "type": "Point",
                            "coordinates": [10.0, 45.0]
                        },
                        {
                            "type": "Point",
                            "coordinates": [20.0, 45.0]
                        }
                    ]
                }
            })
            .to_string(),
        );

        let actual = geojson.to_geo().unwrap();
        let expected = Geometry::GeometryCollection(GeometryCollection::<f64>(vec![
            Point::new(10.0, 45.0).into(),
            Point::new(20.0, 45.0).into(),
        ]));
        assert_eq!(expected, actual);
    }
}
