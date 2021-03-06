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

extern crate nalgebra as na;
extern crate serde_yaml;

use std::collections::HashMap;
use std::f64::consts::PI;
use std::fs;
use std::path::PathBuf;
use std::fs::File;
use std::process;

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

    impl Default for Color {
        fn default() -> Color {
            GROUND
        }
    }

    pub const GROUND:  Color  = Color { value: "#FDD9B5" }; // Sandy Tan
    pub const YELLOW:  Color  = Color { value: "#FDEE00" }; // Aureolin
    pub const GREEN:   Color  = Color { value: "#00A550" }; // Pigment Green
    pub const RUSSET:  Color  = Color { value: "#CD7F32" }; // Bronze
    pub const GREY:    Color  = Color { value: "#ACACAC" }; // Silver Chalice
    pub const BROWN:   Color  = Color { value: "#7B3F00" }; // Chocolate
    pub const RED:     Color  = Color { value: "#DC143C" }; // Crimson
    pub const BLUE:    Color  = Color { value: "#007FFF" }; // Azure
    pub const BARRIER: Color  = Color { value: "#660000" }; // Blood Red
    pub const WHITE:   Color  = Color { value: "#FFFFFF" };

    pub fn name_to_color(name: &String) -> Color {
        match name.to_lowercase().as_str() {
            "ground"  => GROUND,
            "yellow"  => YELLOW,
            "green"   => GREEN,
            "russet"  => RUSSET,
            "grey"    => GREY,
            "brown"   => BROWN,
            "red"     => RED,
            "blue"    => BLUE,
            "barrier" => BARRIER,
            "white"   => WHITE,
            _         => Color { value: "#000000" },
        }
    }
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

/// Converts a compass direction to a number of degrees of rotation
pub fn direction_to_angle(direction: &str) -> f64 {
    match direction {
        "N"  => 0.0,
        "NW" => -PI / 3.0,
        "SW" => -PI * 2.0 / 3.0,
        "S"  => PI,
        "SE" => PI * 2.0 / 3.0,
        "NE" => PI / 3.0,
        c => panic!("Invalid direction {}", c),
    }
}

/// Represents named or hex space coordinate
#[derive(Clone, Deserialize, Debug)]
#[serde(untagged)]
pub enum Coordinate {
    Named(String),
    HexSpace((f64, f64, f64)),
}

impl Coordinate {
    pub fn as_vector(&self) -> na::Vector3<f64> {
        match self {
            &Coordinate::Named(ref name) => edge_to_coordinate(name.as_ref()),
            &Coordinate::HexSpace(ref pos) =>
                na::Vector3::new(pos.0, pos.1, pos.2),
        }
    }
}

/// Attributes that are common between Tile and TileDefinition
pub trait TileSpec {
    fn color(&self) -> colors::Color;
    fn set_name(&mut self, name: String);
    fn name(&self) -> &str;
    /// The paths on the tile.
    fn paths(&self) -> Vec<Path>;
    /// The city revenue locations on the tile.
    fn cities(&self) -> Vec<City>;
    /// The stop revenue locations on the tile
    fn stops(&self) -> Vec<Stop>;
    /// Whether a tile should be drawn as lawson track
    fn is_lawson(&self) -> bool;
    /// Arrows on the edge of a tile
    fn arrows(&self) -> Vec<Coordinate> { vec![] }
    /// Revenue track on the tile
    fn revenue_track(&self) -> Option<RevenueTrack> { None }
    fn terrain(&self) -> Option<Terrain> { None }

    fn get_text<'a>(&'a self, &'a str) -> &'a str;
    fn text_position(&self, usize) -> Option<na::Vector3<f64>>;
    fn text_spec(&self) -> Vec<Text>;

    /// Rotation of the tile
    fn orientation(&self) -> f64 { 0.0 }
}

/// The specification of a tile to be used in the game
#[derive(Deserialize)]
pub struct Tile {
    base_tile: String,
    color: String,
    text: HashMap<String, String>,

    #[serde(skip)]
    definition: Option<TileDefinition>,
}

impl Tile {
    pub fn set_definition(&mut self, definition: &TileDefinition) {
        self.definition = Some(definition.clone());
    }

    pub fn base_tile(&self) -> String {
        self.base_tile.clone()
    }
}

impl Default for Tile {
    fn default() -> Tile {
        Tile {
            base_tile: String::new(),
            color: String::new(),
            text: HashMap::new(),
            definition: None,
        }
    }
}

impl TileSpec for Tile {
    fn color(&self) -> colors::Color {
        colors::name_to_color(&self.color)
    }

    /// The number of the tile, should be the first text specified
    fn name(&self) -> &str {
        self.text.get("number").unwrap()
    }

    fn set_name(&mut self, name: String) {
        self.text.insert("number".to_string(), name);
    }

    fn paths(&self) -> Vec<Path> {
        self.definition.as_ref()
            .expect("You must call set_definition() before using paths()")
            .paths()
    }

    fn cities(&self) -> Vec<City> {
        self.definition.as_ref()
            .expect("You must call set_definition() before using cities()")
            .cities()
    }

    fn stops(&self) -> Vec<Stop> {
        self.definition.as_ref()
            .expect("You must call set_definition() before using stops()")
            .stops()
    }

    fn is_lawson(&self) -> bool {
        self.definition.as_ref()
            .expect("You must call set_definition() before using is_lawson()")
            .is_lawson()
    }

    fn get_text(&self, id: &str) -> &str {
        match self.text.get(id) {
            Some(s) => s,
            None => "",
        }
    }

    fn text_position(&self, id: usize) -> Option<na::Vector3<f64>> {
        self.definition.as_ref()
            .expect("You must call set_definition() before using \
                    text_position()")
            .text_position(id)
    }

    fn text_spec(&self) -> Vec<Text> {
        self.definition.as_ref()
            .expect("You must call set_definition() before using \
                    text_spec()")
            .text_spec()
    }
}

/// Definition of tile layout, does not include color or name
#[derive(Clone, Deserialize, Debug)]
#[serde(default)]
pub struct TileDefinition {
    name: String,
    paths: Vec<Path>,
    cities: Vec<City>,
    stops: Vec<Stop>,
    is_lawson: bool,
    text: Vec<Text>,
}

impl Default for TileDefinition {
    fn default() -> TileDefinition {
        TileDefinition {
            name: "NoName".to_string(),
            paths: vec![],
            cities: vec![],
            stops: vec![],
            is_lawson: false,
            text: vec![],
        }
    }
}

impl TileSpec for TileDefinition {
    fn paths(&self) -> Vec<Path> { self.paths.clone() }
    fn cities(&self) -> Vec<City> { self.cities.clone() }
    fn stops(&self) -> Vec<Stop> { self.stops.clone() }
    fn is_lawson(&self) -> bool { self.is_lawson }
    fn color(&self) -> colors::Color { colors::GROUND }
    fn set_name(&mut self, name: String) { self.name = name; }
    fn name(&self) -> &str { self.name.as_str() }

    fn get_text<'a>(&'a self, id: &'a str) -> &'a str {
        match id {
            "number" => self.name(),
            x => x,
        }
    }

    fn text_position(&self, id: usize) -> Option<na::Vector3<f64>> {
        Some(self.text[id].position())
    }

    fn text_spec(&self) -> Vec<Text> {
        let tile_number = Text {
            id: "number".to_string(),
            position: Coordinate::HexSpace((0.0, 0.0, -0.9)),
            anchor: TextAnchor::End,
            size: None,
            weight: None,
        };
        let mut text = self.text.clone();
        text.insert(0, tile_number);
        text
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
    start: Coordinate,
    end: Coordinate,
    pub start_control: Option<Coordinate>,
    pub end_control: Option<Coordinate>,
    #[serde(default)]
    is_bridge: bool,
}

impl Path {
    /// Getter that always returns the start coordinate in hexagon-space.
    pub fn start(&self) -> na::Vector3<f64> {
        self.start.as_vector()
    }

    /// Getter that always returns the end coordinate in hexagon-space.
    pub fn end(&self) -> na::Vector3<f64> {
        self.end.as_vector()
    }

    /// Whether the is_bridge flag is set
    pub fn is_bridge(&self) -> bool {
        self.is_bridge
    }

    /// The radius of the corner made by the path
    pub fn radius(&self) -> f64 {
        let gentle_curve = 2.0_f64.sqrt() / 2.0;
        // Gentle curves have a different radius
        if let (&Coordinate::Named(ref start), &Coordinate::Named(ref end))
                = (&self.start, &self.end) {
            if start.len() == 2 && end.len() == 2 &&
                start.chars().nth(0) == end.chars().nth(0) {
                // NW-NE, SW-SE
                return gentle_curve
            } else if ((start.len() == 2 && end.len() == 1) ||
                       (start.len() == 1 && end.len() == 2)) &&
                      start.chars().nth(0) != end.chars().nth(0) {
                // N-SE, N-SW, etc.
                return gentle_curve
            }
        }
        // Everything else has a radius of one
        1.0
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
    pub text_id: String,
    pub revenue_position: Coordinate,
    position: Coordinate,
}

impl City {
    /// The coordinate of the city in hexagon-space.
    pub fn position(&self) -> na::Vector3<f64> {
        self.position.as_vector()
    }

    pub fn revenue_position(&self) -> na::Vector3<f64>{
        self.revenue_position.as_vector()
    }
}

/// Stop on the tile
///
/// A stop is a position with a revenue number. The `position` field is an
/// 3D position in hexagon-space.
#[derive(Deserialize, Debug, Clone)]
pub struct Stop {
    position: Coordinate,
    pub text_id: String,
    pub revenue_angle: i32,
}

impl Stop {
    /// The coordinate of the stop in hexagon-space.
    pub fn position(&self) -> na::Vector3<f64> {
        self.position.as_vector()
    }
}

/// Text anchor position for text on tile
#[derive(Deserialize, Debug, Clone)]
pub enum TextAnchor {
    Start,
    Middle,
    End,
}

/// Text on the tile
#[derive(Deserialize, Debug, Clone)]
pub struct Text {
    pub id: String,
    position: Coordinate,
    size: Option<String>,
    pub weight: Option<u32>,
    pub anchor: TextAnchor,
}

impl Text {
    /// The coordinate of the text in hexagon-space.
    pub fn position(&self) -> na::Vector3<f64> {
        self.position.as_vector()
    }

    /// The size of the text
    pub fn size(&self) -> Option<&str> {
        match self.size {
            None => None,
            Some(ref s) => Some(&s),
        }
    }
}

/// Track which shows revenue for different phases
#[derive(Deserialize, Debug, Clone)]
pub struct RevenueTrack {
    position: Coordinate,
    pub yellow: String,
    pub green: Option<String>,
    pub russet: Option<String>,
    pub grey: Option<String>,
}

impl RevenueTrack {
    /// The coordinate of the track in hexagon-space.
    pub fn position(&self) -> na::Vector3<f64> {
        self.position.as_vector()
    }
}

/// Terrain on a tile
#[derive(Clone, Deserialize)]
pub struct Terrain {
    position: Coordinate,
    #[serde(rename="type")]
    pub terrain_type: TerrainType,
    pub cost: String,
}

impl Terrain {
    /// The coordinate of the terrain in hexagon-space.
    pub fn position(&self) -> na::Vector3<f64> {
        self.position.as_vector()
    }
}

/// Types of terrain that can be present
#[derive(Clone, Deserialize)]
#[serde(rename_all="lowercase")]
pub enum TerrainType {
    Rough,
    Hill,
    Mountain,
    River,
    Marsh,
}

/// Reads and parses all tile definitions in ./tiledefs/
pub fn definitions(options: &super::Options)
        -> HashMap<String, TileDefinition> {
    println!("Reading tile definitions from file...");
    let def_files: Vec<PathBuf> = match fs::read_dir("tiledefs") {
        Err(err) => {
            eprintln!("Couldn't open tile definitions directory: {:?}",
                      err.kind());
            process::exit(1);
        }
        Ok(paths) => {
            paths.map(|path| path.unwrap().path()).collect()
        },
    };
    // Read and parse each file
    let mut definitions = HashMap::new();
    for def in &def_files {
        // Ignore non .yaml files
        if def.extension().unwrap() != "yaml" {
            continue;
        }
        if options.verbose {
            println!("Parsing definition {}",
                     def.file_stem().unwrap().to_string_lossy());
        }

        // Read yaml file
        let file = File::open(def).unwrap_or_else(|err| {
            eprintln!("Couldn't open {}: {:?}", def.to_string_lossy(),
                      err.kind());
            process::exit(1);
        });
        let mut tile: TileDefinition = serde_yaml::from_reader(file)
            .unwrap_or_else(|err| {
                eprintln!("Error parsing {}: {}", def.to_string_lossy(), err);
                process::exit(1);
        });
        tile.set_name(String::from(def.file_stem()
                                   .unwrap().to_string_lossy()));
        definitions.insert(String::from(tile.name()), tile);
    }
    definitions
}
