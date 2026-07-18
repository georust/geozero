//! Read MVT features and layers using the `fast-mvt` crate high-level types.

use fast_mvt::{MvtFeature, MvtLayer, MvtValue};
use geo_types::{Coord, Geometry, LineString, Polygon};

use crate::error::Result;
use crate::{ColumnValue, FeatureProcessor, GeomProcessor, GeozeroDatasource, GeozeroGeometry};

impl GeozeroDatasource for MvtLayer {
    fn process<P: FeatureProcessor>(&mut self, processor: &mut P) -> Result<()> {
        process(self, processor)
    }
}

/// Process an MVT layer.
pub fn process(layer: &MvtLayer, processor: &mut impl FeatureProcessor) -> Result<()> {
    processor.dataset_begin(Some(&layer.name))?;
    for (idx, feature) in layer.features.iter().enumerate() {
        processor.feature_begin(idx as u64)?;

        processor.properties_begin()?;
        for (i, (key, value)) in feature.properties.iter().enumerate() {
            process_property(i, key, value, processor)?;
        }
        processor.properties_end()?;

        processor.geometry_begin()?;
        process_geom(feature, processor)?;
        processor.geometry_end()?;

        processor.feature_end(idx as u64)?;
    }
    processor.dataset_end()
}

fn process_property(
    idx: usize,
    key: &str,
    value: &MvtValue,
    processor: &mut impl FeatureProcessor,
) -> Result<()> {
    match value {
        MvtValue::String(v) => processor.property(idx, key, &ColumnValue::String(v))?,
        MvtValue::Float(v) => processor.property(idx, key, &ColumnValue::Float(*v))?,
        MvtValue::Double(v) => processor.property(idx, key, &ColumnValue::Double(*v))?,
        MvtValue::Int(v) | MvtValue::SInt(v) => {
            processor.property(idx, key, &ColumnValue::Long(*v))?
        }
        MvtValue::UInt(v) => processor.property(idx, key, &ColumnValue::ULong(*v))?,
        MvtValue::Bool(v) => processor.property(idx, key, &ColumnValue::Bool(*v))?,
        // `Null` carries no value; nothing to emit.
        MvtValue::Null => false,
    };
    Ok(())
}

impl GeozeroGeometry for MvtFeature {
    fn process_geom<P: GeomProcessor>(&self, processor: &mut P) -> Result<()> {
        process_geom(self, processor)
    }
}

/// Process the geometry of an MVT feature.
pub fn process_geom<P: GeomProcessor>(feature: &MvtFeature, processor: &mut P) -> Result<()> {
    process_geometry(&feature.geometry, 0, processor)
}

fn process_geometry<P: GeomProcessor>(
    geom: &Geometry<i32>,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    match geom {
        Geometry::Point(geom) => {
            processor.point_begin(idx)?;
            process_coord(geom.0, 0, processor)?;
            processor.point_end(idx)
        }
        Geometry::MultiPoint(geom) => {
            processor.multipoint_begin(geom.0.len(), idx)?;
            for (i, pt) in geom.0.iter().enumerate() {
                process_coord(pt.0, i, processor)?;
            }
            processor.multipoint_end(idx)
        }
        Geometry::LineString(geom) => process_linestring(geom, true, idx, processor),
        Geometry::MultiLineString(geom) => {
            processor.multilinestring_begin(geom.0.len(), idx)?;
            for (i, line) in geom.0.iter().enumerate() {
                process_linestring(line, false, i, processor)?;
            }
            processor.multilinestring_end(idx)
        }
        Geometry::Polygon(geom) => process_polygon(geom, true, idx, processor),
        Geometry::MultiPolygon(geom) => {
            processor.multipolygon_begin(geom.0.len(), idx)?;
            for (i, poly) in geom.0.iter().enumerate() {
                process_polygon(poly, false, i, processor)?;
            }
            processor.multipolygon_end(idx)
        }
        Geometry::GeometryCollection(geom) => {
            processor.geometrycollection_begin(geom.0.len(), idx)?;
            for (i, g) in geom.0.iter().enumerate() {
                process_geometry(g, i, processor)?;
            }
            processor.geometrycollection_end(idx)
        }
        // fast-mvt never decodes into these variants
        Geometry::Line(_) | Geometry::Rect(_) | Geometry::Triangle(_) => Ok(()),
    }
}

fn process_coord<P: GeomProcessor>(coord: Coord<i32>, idx: usize, processor: &mut P) -> Result<()> {
    let (x, y) = (f64::from(coord.x), f64::from(coord.y));
    if processor.multi_dim() {
        processor.coordinate(x, y, None, None, None, None, idx)
    } else {
        processor.xy(x, y, idx)
    }
}

fn process_linestring<P: GeomProcessor>(
    geom: &LineString<i32>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    processor.linestring_begin(tagged, geom.0.len(), idx)?;
    for (i, coord) in geom.0.iter().enumerate() {
        process_coord(*coord, i, processor)?;
    }
    processor.linestring_end(tagged, idx)
}

fn process_polygon<P: GeomProcessor>(
    geom: &Polygon<i32>,
    tagged: bool,
    idx: usize,
    processor: &mut P,
) -> Result<()> {
    let interiors = geom.interiors();
    processor.polygon_begin(tagged, interiors.len() + 1, idx)?;
    process_linestring(geom.exterior(), false, 0, processor)?;
    for (i, ring) in interiors.iter().enumerate() {
        process_linestring(ring, false, i + 1, processor)?;
    }
    processor.polygon_end(tagged, idx)
}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use fast_mvt::{DEFAULT_EXTENT, MvtFeature, MvtGeometry, MvtLayer, MvtValue};
    use geo_types::{LineString, MultiPolygon, Point, Polygon, point};
    use serde_json::json;

    use crate::{ProcessToJson, ToJson};

    fn feature(geometry: MvtGeometry) -> MvtFeature {
        MvtFeature {
            id: None,
            geometry,
            properties: Vec::new(),
        }
    }

    #[test]
    fn layer() {
        // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
        let mut mvt_layer = MvtLayer::new("points", DEFAULT_EXTENT);
        mvt_layer.add_feature(MvtFeature {
            id: Some(1),
            geometry: MvtGeometry::Point(point! { x: 1205, y: 1540 }),
            properties: vec![
                ("hello".to_string(), MvtValue::String("world".to_string())),
                ("h".to_string(), MvtValue::String("world".to_string())),
                ("count".to_string(), MvtValue::Double(1.23)),
            ],
        });
        mvt_layer.add_feature(MvtFeature {
            id: Some(2),
            geometry: MvtGeometry::Point(point! { x: 1205, y: 1540 }),
            properties: vec![
                ("hello".to_string(), MvtValue::String("again".to_string())),
                ("count".to_string(), MvtValue::Int(2)),
            ],
        });

        let geojson = mvt_layer.to_json().unwrap();

        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({
                "type": "FeatureCollection",
                "name": "points",
                "features": [
                    {
                        "type": "Feature",
                        "properties": { "hello": "world", "h": "world", "count": 1.23 },
                        "geometry": { "type": "Point", "coordinates": [1205,1540] }
                    },
                    {
                        "type": "Feature",
                        "properties": { "hello": "again", "count": 2 },
                        "geometry": { "type": "Point", "coordinates": [1205,1540] }
                    }
                ]
            })
        );
    }

    #[test]
    fn point_geom() {
        let mvt_feature = feature(MvtGeometry::Point(point! { x: 25, y: 17 }));
        let geojson = mvt_feature.to_json().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({ "type": "Point", "coordinates": [25,17] })
        );
    }

    #[test]
    fn multipoint_geom() {
        let mvt_feature = feature(MvtGeometry::MultiPoint(
            vec![Point::new(5, 7), Point::new(3, 2)].into(),
        ));
        let geojson = mvt_feature.to_json().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({ "type": "MultiPoint", "coordinates": [[5,7],[3,2]] })
        );
    }

    #[test]
    fn line_geom() {
        let mvt_feature = feature(MvtGeometry::LineString(LineString::from(vec![
            (2, 2),
            (2, 10),
            (10, 10),
        ])));
        let geojson = mvt_feature.to_json().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({ "type": "LineString", "coordinates": [[2,2],[2,10],[10,10]] })
        );
    }

    #[test]
    fn polygon_geom() {
        let mvt_feature = feature(MvtGeometry::Polygon(Polygon::new(
            LineString::from(vec![(3, 6), (8, 12), (20, 34), (3, 6)]),
            vec![],
        )));
        let geojson = mvt_feature.to_json().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({ "type": "Polygon", "coordinates": [[[3,6],[8,12],[20,34],[3,6]]] })
        );
    }

    #[test]
    fn multipolygon_geom() {
        let mvt_feature = feature(MvtGeometry::MultiPolygon(MultiPolygon(vec![
            Polygon::new(
                LineString::from(vec![(0, 0), (10, 0), (10, 10), (0, 10), (0, 0)]),
                vec![],
            ),
            Polygon::new(
                LineString::from(vec![(11, 11), (20, 11), (20, 20), (11, 20), (11, 11)]),
                vec![LineString::from(vec![
                    (13, 13),
                    (13, 17),
                    (17, 17),
                    (17, 13),
                    (13, 13),
                ])],
            ),
        ])));
        let geojson = mvt_feature.to_json().unwrap();
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({
                "type": "MultiPolygon",
                "coordinates": [
                    [[[0,0],[10,0],[10,10],[0,10],[0,0]]],
                    [[[11,11],[20,11],[20,20],[11,20],[11,11]],[[13,13],[13,17],[17,17],[17,13],[13,13]]]
                ]
            })
        );
    }
}
