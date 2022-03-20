//! Encode geometries according to MVT spec
//! https://github.com/mapbox/vector-tile-spec/tree/master/2.1

use crate::error::{GeozeroError, Result};
use crate::mvt::mvt_commands::*;
use crate::mvt::vector_tile::{tile, tile::GeomType};
use crate::GeomProcessor;

/// Generator for MVT geometry type.
pub struct MvtWriter {
    pub(crate) feature: tile::Feature,
    last_x: i32,
    last_y: i32,
    line_state: LineState,
    is_multiline: bool,
}

#[derive(PartialEq)]
enum LineState {
    None,
    // Issue LineTo command afer first point
    Line(usize),
    Ring(usize),
}

impl MvtWriter {
    pub fn new() -> MvtWriter {
        MvtWriter {
            feature: tile::Feature::default(),
            last_x: 0,
            last_y: 0,
            line_state: LineState::None,
            is_multiline: false,
        }
    }
    pub fn geometry(&self) -> &tile::Feature {
        &self.feature
    }
}

impl GeomProcessor for MvtWriter {
    fn xy(&mut self, x: f64, y: f64, idx: usize) -> Result<()> {
        // Omit last coord of ring (emit ClosePath instead)
        let last_ring_coord = if let LineState::Ring(size) = self.line_state {
            idx == size - 1
        } else {
            false
        };

        if !last_ring_coord {
            let x = x as i32;
            let y = y as i32;
            self.feature
                .geometry
                .push(ParameterInteger::from(x.saturating_sub(self.last_x)));
            self.feature
                .geometry
                .push(ParameterInteger::from(y.saturating_sub(self.last_y)));
            self.last_x = x;
            self.last_y = y;
        }

        // Emit LineTo command after first coord in line or ring
        if idx == 0 && self.line_state != LineState::None {
            let num_coords = match self.line_state {
                LineState::Line(size) if size > 1 => size - 1,
                LineState::Ring(size) if size > 2 => size - 2,
                _ => {
                    return Err(GeozeroError::Geometry(
                        "Too few coordinates in line or ring".to_string(),
                    ))
                }
            };
            self.feature
                .geometry
                .push(CommandInteger::from(Command::LineTo, num_coords as u32));
        }
        Ok(())
    }
    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.feature.set_type(GeomType::Point);
        self.feature.geometry.reserve(3);
        self.feature
            .geometry
            .push(CommandInteger::from(Command::MoveTo, 1));
        Ok(())
    }
    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.feature.set_type(GeomType::Point);
        self.feature.geometry.reserve(1 + 2 * size);
        self.feature
            .geometry
            .push(CommandInteger::from(Command::MoveTo, size as u32));
        Ok(())
    }
    fn linestring_begin(&mut self, tagged: bool, size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.feature.set_type(GeomType::Linestring);
        }
        self.line_state = if tagged || self.is_multiline {
            self.feature.geometry.reserve(2 + 2 * size); // TODO: correct?
            LineState::Line(size)
        } else {
            self.feature.geometry.reserve(2 + 2 * (size - 1) + 1); // TODO: correct?
            LineState::Ring(size)
        };
        self.feature
            .geometry
            .push(CommandInteger::from(Command::MoveTo, 1));
        Ok(())
    }
    fn linestring_end(&mut self, _tagged: bool, _idx: usize) -> Result<()> {
        if let LineState::Ring(_) = self.line_state {
            self.feature
                .geometry
                .push(CommandInteger::from(Command::ClosePath, 1));
        }
        self.line_state = LineState::None;
        Ok(())
    }
    fn multilinestring_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.is_multiline = true;
        self.feature.set_type(GeomType::Linestring);
        Ok(())
    }
    fn multilinestring_end(&mut self, _size: usize) -> Result<()> {
        self.is_multiline = false;
        Ok(())
    }
    fn polygon_begin(&mut self, tagged: bool, _size: usize, _idx: usize) -> Result<()> {
        if tagged {
            self.feature.set_type(GeomType::Polygon);
        }
        Ok(())
    }
    fn multipolygon_begin(&mut self, _size: usize, _idx: usize) -> Result<()> {
        self.feature.set_type(GeomType::Polygon);
        Ok(())
    }
}

#[cfg(test)]
mod test_mvt {
    use super::*;
    use crate::mvt::vector_tile::Tile;

    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
    const TILE_EXAMPLE: &'static str = r#"Tile {
    layers: [
        Layer {
            version: 2,
            name: "points",
            features: [
                Feature {
                    id: Some(
                        1,
                    ),
                    tags: [
                        0,
                        0,
                        1,
                        0,
                        2,
                        1,
                    ],
                    r#type: Some(
                        Point,
                    ),
                    geometry: [
                        9,
                        490,
                        6262,
                    ],
                },
                Feature {
                    id: Some(
                        2,
                    ),
                    tags: [
                        0,
                        2,
                        2,
                        3,
                    ],
                    r#type: Some(
                        Point,
                    ),
                    geometry: [
                        9,
                        490,
                        6262,
                    ],
                },
            ],
            keys: [
                "hello",
                "h",
                "count",
            ],
            values: [
                Value {
                    string_value: Some(
                        "world",
                    ),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                },
                Value {
                    string_value: None,
                    float_value: None,
                    double_value: Some(
                        1.23,
                    ),
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                },
                Value {
                    string_value: Some(
                        "again",
                    ),
                    float_value: None,
                    double_value: None,
                    int_value: None,
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                },
                Value {
                    string_value: None,
                    float_value: None,
                    double_value: None,
                    int_value: Some(
                        2,
                    ),
                    uint_value: None,
                    sint_value: None,
                    bool_value: None,
                },
            ],
            extent: Some(
                4096,
            ),
        },
    ],
}"#;

    #[test]
    fn test_build_mvt() {
        // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
        let mut mvt_tile = Tile::default();

        let mut mvt_layer = tile::Layer::default();
        mvt_layer.version = 2;
        mvt_layer.name = String::from("points");
        mvt_layer.extent = Some(4096);

        let mut mvt_feature = tile::Feature::default();
        mvt_feature.id = Some(1);
        mvt_feature.set_type(GeomType::Point);
        mvt_feature.geometry = [9, 490, 6262].to_vec();

        let mut mvt_value = tile::Value::default();
        mvt_value.string_value = Some(String::from("world"));
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("hello"),
            mvt_value,
        );
        let mut mvt_value = tile::Value::default();
        mvt_value.string_value = Some(String::from("world"));
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("h"),
            mvt_value,
        );
        let mut mvt_value = tile::Value::default();
        mvt_value.double_value = Some(1.23);
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("count"),
            mvt_value,
        );

        mvt_layer.features.push(mvt_feature);

        mvt_feature = tile::Feature::default();
        mvt_feature.id = Some(2);
        mvt_feature.set_type(GeomType::Point);
        mvt_feature.geometry = [9, 490, 6262].to_vec();

        let mut mvt_value = tile::Value::default();
        mvt_value.string_value = Some(String::from("again"));
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("hello"),
            mvt_value,
        );
        let mut mvt_value = tile::Value::default();
        mvt_value.int_value = Some(2);
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("count"),
            mvt_value,
        );

        mvt_layer.features.push(mvt_feature);

        mvt_tile.layers.push(mvt_layer);
        println!("{:#?}", mvt_tile);
        // Ignore trailing commas because of https://github.com/rust-lang/rust/pull/59076/
        assert_eq!(
            TILE_EXAMPLE.replace(",\n", "\n"),
            &*format!("{:#?}", mvt_tile).replace(",\n", "\n")
        );
    }
}

fn add_feature_attribute(
    mvt_layer: &mut tile::Layer,
    mvt_feature: &mut tile::Feature,
    key: String,
    mvt_value: tile::Value,
) {
    let keyentry = mvt_layer.keys.iter().position(|k| *k == key);
    // Optimization: maintain a hash table with key/index pairs
    let keyidx = match keyentry {
        None => {
            mvt_layer.keys.push(key);
            mvt_layer.keys.len() - 1
        }
        Some(idx) => idx,
    };
    mvt_feature.tags.push(keyidx as u32);

    let valentry = mvt_layer.values.iter().position(|v| *v == mvt_value);
    // Optimization: maintain a hash table with value/index pairs
    let validx = match valentry {
        None => {
            mvt_layer.values.push(mvt_value);
            mvt_layer.values.len() - 1
        }
        Some(idx) => idx,
    };
    mvt_feature.tags.push(validx as u32);
}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::GeoJson;
    use crate::ToMvt;
    use std::convert::TryFrom;

    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#435-example-geometry-encodings

    #[test]
    fn point_geom() {
        let geojson = GeoJson(r#"{"type": "Point", "coordinates": [25, 17]}"#);
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.geometry, [9, 50, 34]);
    }

    #[test]
    fn multipoint_geom() {
        let geojson = GeoJson(r#"{"type": "MultiPoint", "coordinates": [[5, 7], [3, 2]]}"#);
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.geometry, [17, 10, 14, 3, 9]);
    }

    #[test]
    fn line_geom() {
        let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[2,2], [2,10], [10,10]]}"#);
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.geometry, [9, 4, 4, 18, 0, 16, 16, 0]);
    }

    #[test]
    fn multiline_geom() {
        let geojson = GeoJson(
            r#"{"type": "MultiLineString", "coordinates": [[[2,2], [2,10], [10,10]],[[1,1],[3,5]]]}"#,
        );
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(
            mvt.geometry,
            [9, 4, 4, 18, 0, 16, 16, 0, 9, 17, 17, 10, 4, 8]
        );
    }

    #[test]
    fn polygon_geom() {
        let geojson =
            GeoJson(r#"{"type": "Polygon", "coordinates": [[[3, 6], [8, 12], [20, 34], [3, 6]]]}"#);
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(mvt.geometry, [9, 6, 12, 18, 10, 12, 24, 44, 15]);
    }

    #[test]
    fn multipolygon_geom() {
        let geojson = GeoJson(
            r#"{"type": "MultiPolygon", "coordinates": [[[[0,0],[10,0],[10,10],[0,10],[0,0]]],[[[11,11],[20,11],[20,20],[11,20],[11,11]],[[13,13],[13,17],[17,17],[17,13],[13,13]]]]}"#,
        );
        let mvt = geojson.to_mvt().unwrap();
        assert_eq!(
            mvt.geometry,
            [
                9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15, 9, 22, 2, 26, 18, 0, 0, 18, 17, 0, 15, 9, 4,
                13, 26, 0, 8, 8, 0, 0, 7, 15
            ]
        );
    }

    #[test]
    #[cfg(feature = "with-geo")]
    fn geo_to_mvt() -> Result<()> {
        let geo =
            geo_types::Geometry::try_from(wkt::Wkt::from_str("POINT (25 17)").unwrap()).unwrap();
        let mvt = geo.to_mvt()?;
        assert_eq!(mvt.geometry, [9, 50, 34]);
        Ok(())
    }
}
