//! Encode geometries according to MVT spec
//! <https://github.com/mapbox/vector-tile-spec/tree/master/2.1>

use crate::error::Result;
use crate::mvt::mvt_commands::{Command, CommandInteger, ParameterInteger};
use crate::mvt::vector_tile::{tile, tile::GeomType};
use crate::GeomProcessor;

use super::mvt_error::MvtError;

/// Generator for MVT geometry type.
#[derive(Default, Debug)]
pub struct MvtWriter {
    pub(crate) feature: tile::Feature,
    // Extent, 0 for unscaled
    extent: i32,
    // Scale geometry to bounds
    left: f64,
    bottom: f64,
    x_multiplier: f64,
    y_multiplier: f64,
    // Writer state
    last_x: i32,
    last_y: i32,
    line_state: LineState,
    is_multiline: bool,
}

#[derive(Default, Debug, PartialEq)]
enum LineState {
    #[default]
    None,
    // Issue LineTo command after first point
    Line(usize),
    Ring(usize),
}

impl MvtWriter {
    pub fn new(extent: u32, left: f64, bottom: f64, right: f64, top: f64) -> MvtWriter {
        assert_ne!(extent, 0);
        MvtWriter {
            extent: extent as i32,
            left,
            bottom,
            x_multiplier: (extent as f64) / (right - left),
            y_multiplier: (extent as f64) / (top - bottom),
            ..Default::default()
        }
    }

    pub fn geometry(&self) -> &tile::Feature {
        &self.feature
    }

    fn reserve(&mut self, capacity: usize) {
        let total = self.feature.geometry.len() + capacity;
        if total > self.feature.geometry.capacity() {
            self.feature
                .geometry
                .reserve(total - self.feature.geometry.capacity());
        }
    }
}

impl GeomProcessor for MvtWriter {
    fn xy(&mut self, x_coord: f64, y_coord: f64, idx: usize) -> Result<()> {
        // Omit last coord of ring (emit ClosePath instead)
        let last_ring_coord = if let LineState::Ring(size) = self.line_state {
            idx == size - 1
        } else {
            false
        };

        if !last_ring_coord {
            let (x, y) = if self.extent != 0 {
                // scale to tile coordinate space
                let x = ((x_coord - self.left) * self.x_multiplier) as i32;
                let y = ((y_coord - self.bottom) * self.y_multiplier) as i32;
                // Y is stored as reversed
                (x, self.extent.saturating_sub(y))
            } else {
                // unscaled
                (x_coord as i32, y_coord as i32)
            };
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
                _ => return Err(MvtError::TooFewCoordinates)?,
            };
            self.feature
                .geometry
                .push(CommandInteger::from(Command::LineTo, num_coords as u32));
        }
        Ok(())
    }

    fn point_begin(&mut self, _idx: usize) -> Result<()> {
        self.feature.set_type(GeomType::Point);
        self.reserve(3);
        self.feature
            .geometry
            .push(CommandInteger::from(Command::MoveTo, 1));
        Ok(())
    }

    fn multipoint_begin(&mut self, size: usize, _idx: usize) -> Result<()> {
        self.feature.set_type(GeomType::Point);
        self.reserve(1 + 2 * size);
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
            self.reserve(2 + 2 * size);
            LineState::Line(size)
        } else {
            self.reserve(2 + 2 * (size - 1) + 1);
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
    use crate::mvt::TileValue;

    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#45-example
    const TILE_EXAMPLE: &str = r#"Tile {
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

        let mut mvt_layer = tile::Layer {
            version: 2,
            ..Default::default()
        };
        mvt_layer.name = String::from("points");
        mvt_layer.extent = Some(4096);

        let mut mvt_feature = tile::Feature {
            id: Some(1),
            ..Default::default()
        };
        mvt_feature.set_type(GeomType::Point);
        mvt_feature.geometry = [9, 490, 6262].to_vec();

        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("hello"),
            TileValue::Str("world".to_string()),
        );
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("h"),
            TileValue::Str("world".to_string()),
        );
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("count"),
            TileValue::Double(1.23),
        );

        mvt_layer.features.push(mvt_feature);

        mvt_feature = tile::Feature::default();
        mvt_feature.id = Some(2);
        mvt_feature.set_type(GeomType::Point);
        mvt_feature.geometry = [9, 490, 6262].to_vec();

        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("hello"),
            TileValue::Str("again".to_string()),
        );
        add_feature_attribute(
            &mut mvt_layer,
            &mut mvt_feature,
            String::from("count"),
            TileValue::Int(2),
        );

        mvt_layer.features.push(mvt_feature);
        mvt_tile.layers.push(mvt_layer);

        // println!("{mvt_tile:#?}");
        // Ignore trailing commas because of https://github.com/rust-lang/rust/pull/59076/
        assert_eq!(
            TILE_EXAMPLE.replace(",\n", "\n"),
            &*format!("{mvt_tile:#?}").replace(",\n", "\n")
        );
    }

    fn add_feature_attribute(
        mvt_layer: &mut tile::Layer,
        mvt_feature: &mut tile::Feature,
        key: String,
        value: TileValue,
    ) {
        let mvt_value = value.into();
        let key_entry = mvt_layer.keys.iter().position(|k| *k == key);
        // Optimization: maintain a hash table with key/index pairs
        let key_idx = match key_entry {
            None => {
                mvt_layer.keys.push(key);
                mvt_layer.keys.len() - 1
            }
            Some(idx) => idx,
        };
        mvt_feature.tags.push(key_idx as u32);

        let val_entry = mvt_layer.values.iter().position(|v| *v == mvt_value);
        // Optimization: maintain a hash table with value/index pairs
        let validx = match val_entry {
            None => {
                mvt_layer.values.push(mvt_value);
                mvt_layer.values.len() - 1
            }
            Some(idx) => idx,
        };
        mvt_feature.tags.push(validx as u32);
    }
}

#[cfg(test)]
#[cfg(feature = "with-geojson")]
mod test {
    use super::*;
    use crate::geojson::conversion::ToJson;
    use crate::geojson::GeoJson;
    use crate::ToMvt;
    use serde_json::json;

    // https://github.com/mapbox/vector-tile-spec/tree/master/2.1#435-example-geometry-encodings

    #[test]
    fn point_geom() {
        let geojson = GeoJson(r#"{"type": "Point", "coordinates": [25, 17]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(mvt.geometry, [9, 50, 34]);
    }

    #[test]
    fn multipoint_geom() {
        let geojson = GeoJson(r#"{"type": "MultiPoint", "coordinates": [[5, 7], [3, 2]]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(mvt.geometry, [17, 10, 14, 3, 9]);
    }

    #[test]
    fn line_geom() {
        let geojson = GeoJson(r#"{"type": "LineString", "coordinates": [[2,2], [2,10], [10,10]]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(mvt.geometry, [9, 4, 4, 18, 0, 16, 16, 0]);
    }

    #[test]
    fn multiline_geom() {
        let geojson = GeoJson(
            r#"{"type": "MultiLineString", "coordinates": [[[2,2], [2,10], [10,10]],[[1,1],[3,5]]]}"#,
        );
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            [9, 4, 4, 18, 0, 16, 16, 0, 9, 17, 17, 10, 4, 8]
        );
    }

    #[test]
    fn polygon_geom() {
        let geojson =
            GeoJson(r#"{"type": "Polygon", "coordinates": [[[3, 6], [8, 12], [20, 34], [3, 6]]]}"#);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(mvt.geometry, [9, 6, 12, 18, 10, 12, 24, 44, 15]);
    }

    #[test]
    fn multipolygon_geom() {
        let geojson = r#"{
            "type": "MultiPolygon",
            "coordinates": [
                [
                    [
                        [0,0],[10,0],[10,10],[0,10],[0,0]
                    ]
                ],[
                    [
                        [11,11],[20,11],[20,20],[11,20],[11,11]
                    ],[
                        [13,13],[13,17],[17,17],[17,13],[13,13]
                    ]
                ]
            ]
        }"#;
        let geojson = GeoJson(geojson);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            [
                9, 0, 0, 26, 20, 0, 0, 20, 19, 0, 15, 9, 22, 2, 26, 18, 0, 0, 18, 17, 0, 15, 9, 4,
                13, 26, 0, 8, 8, 0, 0, 7, 15
            ]
        );
    }
    #[test]
    fn big_number_geom() {
        let geojson = r#"{
            "type": "Polygon",
            "coordinates": [[[34876,37618],[37047,39028],[37756,39484],[38779,40151],[39247,40451],[39601,40672],[40431,41182],[41010,41525],[41834,41995],[42190,42193],[42547,42387],[42540,42402],[42479,42516],[42420,42627],[42356,42749],[42344,42770],[42337,42784],[41729,42461],[40755,41926],[40118,41563],[39435,41161],[38968,40882],[38498,40595],[37200,39786],[36547,39382],[34547,38135],[34555,38122],[34595,38059],[34655,37964],[34726,37855],[34795,37745],[34863,37638],[34876,37618]]]
        }"#;
        let geojson = GeoJson(geojson);
        let mvt = geojson.to_mvt_unscaled().unwrap();
        assert_eq!(
            mvt.geometry,
            [
                9, 69752, 75236, 250, 4342, 2820, 1418, 912, 2046, 1334, 936, 600, 708, 442, 1660,
                1020, 1158, 686, 1648, 940, 712, 396, 714, 388, 13, 30, 121, 228, 117, 222, 127,
                244, 23, 42, 13, 28, 1215, 645, 1947, 1069, 1273, 725, 1365, 803, 933, 557, 939,
                573, 2595, 1617, 1305, 807, 3999, 2493, 16, 25, 80, 125, 120, 189, 142, 217, 138,
                219, 136, 213, 15,
            ]
        );
    }

    #[test]
    #[cfg(feature = "with-geo")]
    fn geo_screen_coords_to_mvt() -> Result<()> {
        let geo: geo_types::Geometry<f64> = geo_types::Point::new(25.0, 17.0).into();
        let mvt = geo.to_mvt_unscaled()?;
        assert_eq!(mvt.geometry, [9, 50, 34]);
        Ok(())
    }

    #[test]
    #[cfg(feature = "with-geo")]
    fn geo_to_mvt() -> Result<()> {
        let geo: geo_types::Geometry<f64> = geo_types::Point::new(960000.0, 6002729.0).into();
        let mvt = geo.to_mvt(256, 958826.08, 5987771.04, 978393.96, 6007338.92)?;
        assert_eq!(mvt.geometry, [9, 30, 122]);
        let geojson = mvt.to_json()?;
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&geojson).unwrap(),
            json!({
                "type": "Point",
                "coordinates": [15,61]
            }) // without reverse_y: [15,195]
        );
        Ok(())
    }
}
