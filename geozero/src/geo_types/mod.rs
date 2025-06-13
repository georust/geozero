//! geo-types conversions.
mod geo_types_feature_writer;
pub(crate) mod geo_types_reader;
pub(crate) mod geo_types_writer;

pub use geo_types_feature_writer::*;
pub use geo_types_reader::*;
pub use geo_types_writer::*;

pub(crate) mod conversion {
    use crate::error::{GeozeroError, Result};
    use crate::geo_types::GeoWriter;
    use crate::geo_types::geo_types_feature_writer::{GeoFeature, GeoFeatureWriter};
    use crate::{GeozeroDatasource, GeozeroGeometry};

    /// Convert to geo-types Geometry.
    pub trait ToGeo {
        /// Convert to geo-types Geometry.
        fn to_geo(&self) -> Result<geo_types::Geometry<f64>>;
    }

    impl<T: GeozeroGeometry> ToGeo for T {
        fn to_geo(&self) -> Result<geo_types::Geometry<f64>> {
            let mut geo = GeoWriter::new();
            self.process_geom(&mut geo)?;
            geo.take_geometry()
                .ok_or(GeozeroError::Geometry("Missing Geometry".to_string()))
        }
    }

    pub trait ToGeoFeatures {
        fn to_geo_features(&mut self) -> Result<impl Iterator<Item = GeoFeature>>;
    }

    impl<DS: GeozeroDatasource> ToGeoFeatures for DS {
        fn to_geo_features(&mut self) -> Result<impl Iterator<Item = GeoFeature>> {
            let mut geo = GeoFeatureWriter::new();
            self.process(&mut geo)?;
            Ok(geo.features.into_iter())
        }
    }
}

#[cfg(feature = "with-wkb")]
mod wkb {
    use crate::error::{GeozeroError, Result};
    use crate::geo_types::GeoWriter;
    use crate::wkb::{FromWkb, WkbDialect};
    use std::io::Read;

    impl FromWkb for geo_types::Geometry<f64> {
        fn from_wkb<R: Read>(rdr: &mut R, dialect: WkbDialect) -> Result<Self> {
            let mut geo = GeoWriter::new();
            crate::wkb::process_wkb_type_geom(rdr, &mut geo, dialect)?;
            geo.take_geometry()
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
