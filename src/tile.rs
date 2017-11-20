extern crate toml;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::fs::File;
use std::io::prelude::*;

pub mod colors {
    pub struct Color {
        value: &'static str,
    }

    impl Color {
        pub fn value(&self) -> &str {
            self.value
        }
    }

    pub const GROUND:  Color  = Color { value: "#F5F5F5" };
    pub const YELLOW:  Color  = Color { value: "#FFFF00" };
    pub const GREEN:   Color  = Color { value: "#64E164" };
    pub const RUSSET:  Color  = Color { value: "#EE7621" };
    pub const GREY:    Color  = Color { value: "#BEBEBE" };
    pub const BROWN:   Color  = Color { value: "#CD6600" };
    pub const RED:     Color  = Color { value: "#FF6464" };
    pub const BLUE:    Color  = Color { value: "#6464FF" };
    pub const BARRIER: Color  = Color { value: "#1E90FF" };
    pub const WHITE:   Color  = Color { value: "#FFFFFF" };
}

fn edge_to_coordinate(edge: &str) -> na::Point3<f64> {
    match edge {
        "N"  => na::Point3::new( 0.0,  0.5, -0.5),
        "NE" => na::Point3::new( 0.5,  0.0, -0.5),
        "SE" => na::Point3::new( 0.5, -0.5,  0.0),
        "S"  => na::Point3::new( 0.0, -0.5,  0.5),
        "SW" => na::Point3::new(-0.5,  0.0,  0.5),
        "NW" => na::Point3::new(-0.5,  0.5,  0.0),
        "C"  => na::Point3::new( 0.0,  0.0,  0.0),
        c => panic!("Invalid edge code {}", c),
    }
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

#[derive(Deserialize, Debug)]
pub struct TileDefinition {
    path: Option<Vec<Path>>,
    city: Option<Vec<City>>,
}

impl TileDefinition {
    fn paths(self) -> Option<Vec<Path>> {
        self.path
    }

    fn cities(self) -> Option<Vec<City>> {
        self.city
    }
}

#[derive(Deserialize, Debug)]
pub struct Path {
    start: Option<String>,
    start_pos: Option<Box<[f64]>>,
    end: Option<String>,
    end_pos: Option<Box<[f64]>>,
    stops: Option<u32>,
    revenue: Option<u32>,
}

impl Path {
    pub fn start(&self) -> na::Point3<f64> {
        match &self.start_pos {
            &Some(ref pos) => na::Point3::new(pos[0], pos[1], pos[2]),
            &None => match &self.start {
                &Some(ref s) => edge_to_coordinate(s.as_ref()),
                &None => panic!("No start position found"),
            }
        }
    }

    pub fn end(&self) -> na::Point3<f64> {
        match &self.end_pos {
            &Some(ref pos) => na::Point3::new(pos[0], pos[1], pos[2]),
            &None => match &self.end {
                &Some(ref s) => edge_to_coordinate(s.as_ref()),
                _ => panic!("No end position found"),
            }
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct City {
    circles: u32,
    revenue: u32,
    position: Option<String>,
    pos: Option<Box<[f64]>>,
}

impl City {
    pub fn position(&self) -> na::Point3<f64> {
        match &self.pos {
            &Some(ref pos) => na::Point3::new(pos[0], pos[1], pos[2]),
            &None => match &self.position {
                &Some(ref s) => edge_to_coordinate(s.as_ref()),
                &None => na::Point3::new(0.0, 0.0, 0.0),
            }
        }
    }
}

/// Reads and parses all tile definitions in ./tiledefs/
pub fn definitions() -> HashMap<String, TileDefinition> {
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

        println!("Parsing tile definition {}...",
                 def.file_stem().unwrap().to_str().unwrap());
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
    fn city_position_is_center_by_default() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            "#).unwrap();
        assert_eq!(tile.cities().unwrap()[0].position(),
                   na::Point3::new(0.0, 0.0, 0.0));
    }

    #[test]
    fn city_returns_pos_when_position_also_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            pos = [0.3, 0.0, -0.3]
            "#).unwrap();
        assert_eq!(tile.cities().unwrap()[0].position(),
                   na::Point3::new(0.3, 0.0, -0.3));
    }

    #[test]
    fn city_returns_edge_position_when_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            position = "N"
            "#).unwrap();
        assert_eq!(tile.cities().unwrap()[0].position(),
                   na::Point3::new(0.0, 0.5, -0.5));
    }

    #[test]
    fn city_returns_center_when_position_is_c() {
        let tile: TileDefinition = toml::from_str(r#"
            [[city]]
            circles = 1
            revenue = 10
            position = "C"
            "#).unwrap();
        assert_eq!(tile.cities().unwrap()[0].position(),
                   na::Point3::new(0.0, 0.0, 0.0));
    }

    #[test]
    #[should_panic(expected = "No start position found")]
    fn path_panics_on_no_start_found() {
        let tile: TileDefinition = toml::from_str("[[path]]").unwrap();
        tile.paths().unwrap()[0].start();
    }

    #[test]
    #[should_panic(expected = "Invalid edge code A")]
    fn path_panics_on_start_invalid_code() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "A"}]"#).unwrap();
        tile.paths().unwrap()[0].start();
    }

    #[test]
    #[should_panic(expected = "No end position found")]
    fn path_panics_on_no_end_found() {
        let tile: TileDefinition = toml::from_str("[[path]]").unwrap();
        tile.paths().unwrap()[0].end();
    }

    #[test]
    #[should_panic(expected = "Invalid edge code B")]
    fn path_panics_on_end_invalid_code() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{end = "B"}]"#).unwrap();
        tile.paths().unwrap()[0].end();
    }

    #[test]
    fn path_start_returns_start_pos_when_both_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[path]]
            start_pos = [-0.1, 0.1, 0.0]
            start = "N"
            "#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(-0.1_f64, 0.1, 0.0));
    }

    #[test]
    fn path_converts_start_pos() {
        let tile: TileDefinition = toml::from_str(r#"
        [[path]]
        start_pos = [0.1, 0.3, 0.7]
        "#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(0.1_f64, 0.3, 0.7));
    }

    #[test]
    fn path_converts_start_n() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "N"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(0.0, 0.5, -0.5));
    }

    #[test]
    fn path_converts_start_ne() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "NE"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(0.5, 0.0, -0.5));
    }

    #[test]
    fn path_converts_start_nw() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "NW"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(-0.5, 0.5, 0.0));
    }

    #[test]
    fn path_converts_start_s() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "S"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(0.0, -0.5, 0.5));
    }

    #[test]
    fn path_converts_start_se() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "SE"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(0.5, -0.5, 0.0));
    }

    #[test]
    fn path_converts_start_sw() {
        let tile: TileDefinition = toml::from_str(
            r#"path = [{start = "SW"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].start(),
                   na::Point3::new(-0.5, 0.0, 0.5));
    }

    #[test]
    fn path_end_returns_end_pos_when_both_given() {
        let tile: TileDefinition = toml::from_str(r#"
            [[path]]
            end_pos = [0.5, 0.5, 0.5]
            end = "N"
            "#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(0.5_f64, 0.5, 0.5));
    }

    #[test]
    fn path_converts_end_pos() {
        let tile: TileDefinition = toml::from_str(r#"
            [[path]]
            end_pos = [0.2, 0.4, 0.6]
            "#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(0.2_f64, 0.4, 0.6));
    }

    #[test]
    fn path_converts_end_n() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "N"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(0.0, 0.5, -0.5));
    }

    #[test]
    fn path_converts_end_ne() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "NE"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(0.5, 0.0, -0.5));
    }

    #[test]
    fn path_converts_end_nw() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "NW"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(-0.5, 0.5, 0.0));
    }

    #[test]
    fn path_converts_end_s() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "S"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(0.0, -0.5, 0.5));
    }

    #[test]
    fn path_converts_end_se() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "SE"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(0.5, -0.5, 0.0));
    }

    #[test]
    fn path_converts_end_sw() {
        let tile:TileDefinition = toml::from_str(
            r#"path = [{end = "SW"}]"#).unwrap();
        assert_eq!(tile.paths().unwrap()[0].end(),
                   na::Point3::new(-0.5, 0.0, 0.5));
    }
}
