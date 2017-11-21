//! Representation of tiles
//!
//! Items within a hex are usually given in hexagon-space. This is a 3D space
//! where the axis are at 60° to each other. An example of the axis is given
//! below. Note that the orientation of the axis when the hexagons are oriented
//! with horizontal edges differs from when the hexagons are oriented with
//! vertical edges.
//!
//! Instead of using coordinates in hexagon-space there are these position
//! codes that can be used as a shortcut. North is the upper edge on a hexagon
//! that has horizontal edges, it is the top left edge on hexagons that are
//! oriented vertically.
//!
//! * `N`:  north edge
//! * `NE`: north east edge
//! * `NW`: north west edge
//! * `S`:  south edge
//! * `SE`: south east edge
//! * `SW`: south west edge
//! * `C`:  center of hexagon
//!
//! ![Coordinate system](../../../../axes.svg)

extern crate toml;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

/// Standard colors that can be used
pub mod colors {
    pub struct Color {
        value: &'static str,
    }

    impl Color {
        pub fn value(&self) -> &str {
            self.value
        }
    }

    pub const GROUND:  Color  = Color { value: "#FDD9B5" }; // Sandy Tan
    pub const YELLOW:  Color  = Color { value: "#FDEE00" }; // Aureolin
    pub const GREEN:   Color  = Color { value: "#7CFC00" }; // Lawn Green
    pub const RUSSET:  Color  = Color { value: "#CD7F32" }; // Bronze
    pub const GREY:    Color  = Color { value: "#ACACAC" }; // Silver Chalice
    pub const BROWN:   Color  = Color { value: "#7B3F00" }; // Chocolate
    pub const RED:     Color  = Color { value: "#C80815" }; // Venetian Red
    pub const BLUE:    Color  = Color { value: "#007FFF" }; // Azure
    pub const BARRIER: Color  = Color { value: "#660000" }; // Blood Red
    pub const WHITE:   Color  = Color { value: "#FFFFFF" };
}

/// Converts a position code to hex coordinates
///
/// Converts a position code to a hexagon-space coordinate with its origin in
/// the hexagon center.
///
/// # Panics
///
/// On invalid position code
fn edge_to_coordinate(edge: &str) -> na::Vector3<f64> {
    match edge {
        "N"  => na::Vector3::new( 0.0,  0.5,  0.5),
        "NE" => na::Vector3::new( 0.5,  0.5,  0.0),
        "SE" => na::Vector3::new( 0.5,  0.0, -0.5),
        "S"  => na::Vector3::new( 0.0, -0.5, -0.5),
        "SW" => na::Vector3::new(-0.5, -0.5,  0.0),
        "NW" => na::Vector3::new(-0.5,  0.0,  0.5),
        "C"  => na::Vector3::new( 0.0,  0.0,  0.0),
        c => panic!("Invalid edge code {}", c),
    }
}

/// Attributes that are common between Tile and TileDefinition
pub trait TileSpec {
    fn color(&self) -> colors::Color;
}

pub struct Tile {
    number: String,
    color: colors::Color,
}

impl Tile {
    pub fn new(number: String, color: colors::Color) -> Tile {
        Tile { number, color }
    }

    pub fn color(&self) -> &str {
        self.color.value()
    }
}

/// Definition of tile layout, does not include color or name
#[derive(Deserialize, Debug)]
pub struct TileDefinition {
    path: Option<Vec<Path>>,
    city: Option<Vec<City>>,
    stop: Option<Vec<Stop>>,
}

impl TileDefinition {
    /// The paths on the tile.
    pub fn paths(&self) -> Vec<Path> {
        match self.path {
            Some(ref paths) => paths.to_vec(),
            None => vec![],
        }
    }

    /// The city revenue locations on the tile.
    pub fn cities(&self) -> Vec<City> {
        match self.city {
            Some(ref cities) => cities.to_vec(),
            None => vec![],
        }
    }

    /// The stop revenue locations on the tile
    pub fn stops(&self) -> Vec<Stop> {
        match self.stop {
            Some(ref stops) => stops.to_vec(),
            None => vec![],
        }
    }
}

impl TileSpec for TileDefinition {
    fn color(&self) -> colors::Color {
        colors::GROUND
    }
}

/// Path on the tile
///
/// A path is a line section that goes between `start point` and `end point`.
/// There are two versions of each point `[start|end]` and `[start|end]_pos`,
/// the `_pos` variant takes precedence over the non-`_pos` version. The
/// non-`_pos` version should always be a position code, while the `_pos`
/// version is a 3D position in hexagon-space.
#[derive(Deserialize, Debug, Clone)]
pub struct Path {
    start: Option<String>,
    start_pos: Option<Box<[f64]>>,
    end: Option<String>,
    end_pos: Option<Box<[f64]>>,
}

impl Path {
    /// Getter that always returns the start coordinate in hexagon-space.
    pub fn start(&self) -> na::Vector3<f64> {
        match &self.start_pos {
            &Some(ref pos) => na::Vector3::new(pos[0], pos[1], pos[2]),
            &None => match &self.start {
                &Some(ref s) => edge_to_coordinate(s.as_ref()),
                &None => panic!("No start position found"),
            }
        }
    }

    /// Getter that always returns the end coordinate in hexagon-space.
    pub fn end(&self) -> na::Vector3<f64> {
        match &self.end_pos {
            &Some(ref pos) => na::Vector3::new(pos[0], pos[1], pos[2]),
            &None => match &self.end {
                &Some(ref s) => edge_to_coordinate(s.as_ref()),
                _ => panic!("No end position found"),
            }
        }
    }
}

/// City on the tile
///
/// A city is a collection of circles where tokens can be put down. A city
/// requires the specification of the number of circles (a positive integer)
/// and the revenue (a positive integer). An optional position can also be
/// given. If omitted then the position is assumed to be the center of the
/// tile. The position can be given as the `pos` or `position` fields. The
/// `pos` field is a coordinate in hexagon-space. The `position` field is a
/// position code.
#[derive(Deserialize, Debug, Clone)]
pub struct City {
    pub circles: u32,
    pub revenue: u32,
    position: Option<String>,
    pos: Option<Box<[f64]>>,
}

impl City {
    /// The coordinate of the city in hexagon-space.
    pub fn position(&self) -> na::Vector3<f64> {
        match &self.pos {
            &Some(ref pos) => na::Vector3::new(pos[0], pos[1], pos[2]),
            &None => match &self.position {
                &Some(ref s) => edge_to_coordinate(s.as_ref()),
                &None => na::Vector3::new(0.0, 0.0, 0.0),
            }
        }
    }
}

/// Stop on the tile
///
/// A stop is a position with a revenue number. The `position` field is an
/// 3D position in hexagon-space.
#[derive(Deserialize, Debug, Clone)]
pub struct Stop {
    position: Box<[f64]>,
    revenue: u32,
}

impl Stop {
    /// The coordinate of the stop in hexagon-space.
    pub fn position(&self) -> na::Vector3<f64> {
        na::Vector3::new(self.position[0], self.position[1], self.position[2])
    }
}

/// Reads and parses all tile definitions in ./tiledefs/
pub fn definitions() -> HashMap<String, TileDefinition> {
    println!("Reading tile definitions from file...");
    let def_files: Vec<PathBuf> = match fs::read_dir("tiledefs") {
        Err(why) => panic!("! {:?}", why.kind()),
        Ok(paths) => {
            paths.map(|path| path.unwrap().path()).collect()
        },
    };
    // Read and parse each file
    let mut definitions = HashMap::new();
    for def in &def_files {
        // Ignore non .toml files
        if def.extension().unwrap() != "toml" {
            continue;
        }

        // Read TOML file
        let mut file = File::open(def).unwrap();
        let mut contents = String::new();
        file.read_to_string(&mut contents).unwrap();

        // Parse TOML file
        let tile: TileDefinition = toml::from_str(&contents).unwrap();
        definitions.insert(String::from(def.file_stem().unwrap()
                                           .to_string_lossy()),
                           tile);
    }
    definitions
}

#[cfg(test)]
mod tests {
    extern crate toml;
    use super::{TileDefinition, na};

    #[test]
    fn stop_position_returns_vector3() {
        let tile: TileDefinition = toml::from_str(r#"
            [[stop]]
            position = [0.1, 0.2, 0.4]
            revenue = 10
            "#).unwrap();
        assert_eq!(tile.stops()[0].position(),
                   na::Vector3::new(0.1, 0.2, 0.4));
    }

    #[test]
    fn city_position_is_center_by_default() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            "#).unwrap();
        assert_eq!(tile.cities()[0].position(),
                   na::Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn city_returns_pos_when_position_also_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            pos = [0.3, 0.0, -0.3]
            "#).unwrap();
        assert_eq!(tile.cities()[0].position(),
                   na::Vector3::new(0.3, 0.0, -0.3));
    }

    #[test]
    fn city_returns_edge_position_when_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            position = "N"
            "#).unwrap();
        assert_eq!(tile.cities()[0].position(),
                   na::Vector3::new(0.0, 0.5, 0.5));
    }

    #[test]
    fn city_returns_center_when_position_is_c() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            position = "C"
            "#).unwrap();
        assert_eq!(tile.cities()[0].position(),
                   na::Vector3::new(0.0, 0.0, 0.0));
    }

    #[test]
    #[should_panic(expected = "No start position found")]
    fn path_panics_on_no_start_found() {
        let tile: TileDefinition = toml::from_str("[[path]]").unwrap();
        tile.paths()[0].start();
    }

    #[test]
    #[should_panic(expected = "Invalid edge code A")]
    fn path_panics_on_start_invalid_code() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "A"}]"#).unwrap();
        tile.paths()[0].start();
    }

    #[test]
    #[should_panic(expected = "No end position found")]
    fn path_panics_on_no_end_found() {
        let tile: TileDefinition = toml::from_str("[[path]]").unwrap();
        tile.paths()[0].end();
    }

    #[test]
    #[should_panic(expected = "Invalid edge code B")]
    fn path_panics_on_end_invalid_code() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{end = "B"}]"#).unwrap();
        tile.paths()[0].end();
    }

    #[test]
    fn path_start_returns_start_pos_when_both_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[path]]
            start_pos = [-0.1, 0.1, 0.0]
            start = "N"
            "#).unwrap();
        assert_eq!(tile.paths()[0].start(),
                   na::Vector3::new(-0.1_f64, 0.1, 0.0));
    }

    #[test]
    fn path_converts_start_pos() {
        let tile: TileDefinition = toml::from_str(r#"
        [[path]]
        start_pos = [0.1, 0.3, 0.7]
        "#).unwrap();
        assert_eq!(tile.paths()[0].start(), na::Vector3::new(0.1, 0.3, 0.7));
    }

    #[test]
    fn path_converts_start_n() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "N"}]"#).unwrap();
        assert_eq!(tile.paths()[0].start(), na::Vector3::new(0.0, 0.5, 0.5));
    }

    #[test]
    fn path_converts_start_ne() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "NE"}]"#).unwrap();
        assert_eq!(tile.paths()[0].start(), na::Vector3::new(0.5, 0.5, 0.0));
    }

    #[test]
    fn path_converts_start_nw() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "NW"}]"#).unwrap();
        assert_eq!(tile.paths()[0].start(), na::Vector3::new(-0.5, 0.0, 0.5));
    }

    #[test]
    fn path_converts_start_s() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "S"}]"#).unwrap();
        assert_eq!(tile.paths()[0].start(), na::Vector3::new(0.0, -0.5, -0.5));
    }

    #[test]
    fn path_converts_start_se() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "SE"}]"#).unwrap();
        assert_eq!(tile.paths()[0].start(), na::Vector3::new(0.5, 0.0, -0.5));
    }

    #[test]
    fn path_converts_start_sw() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "SW"}]"#).unwrap();
        assert_eq!(tile.paths()[0].start(), na::Vector3::new(-0.5, -0.5, 0.0));
    }

    #[test]
    fn path_end_returns_end_pos_when_both_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[path]]
            end_pos = [0.5, 0.5, 0.5]
            end = "N"
            "#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(0.5_f64, 0.5, 0.5));
    }

    #[test]
    fn path_converts_end_pos() {
        let tile: TileDefinition = toml::from_str(r#"
            [[path]]
            end_pos = [0.2, 0.4, 0.6]
            "#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(0.2_f64, 0.4, 0.6));
    }

    #[test]
    fn path_converts_end_n() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "N"}]"#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(0.0, 0.5, 0.5));
    }

    #[test]
    fn path_converts_end_ne() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "NE"}]"#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(0.5, 0.5, 0.0));
    }

    #[test]
    fn path_converts_end_nw() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "NW"}]"#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(-0.5, 0.0, 0.5));
    }

    #[test]
    fn path_converts_end_s() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "S"}]"#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(0.0, -0.5, -0.5));
    }

    #[test]
    fn path_converts_end_se() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "SE"}]"#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(0.5, 0.0, -0.5));
    }

    #[test]
    fn path_converts_end_sw() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "SW"}]"#).unwrap();
        assert_eq!(tile.paths()[0].end(), na::Vector3::new(-0.5, -0.5, 0.0));
    }
}
