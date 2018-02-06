extern crate nalgebra as na;
extern crate serde_json;

use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;
use std::process;

use tile;
use tile::TileSpec;

/// Orientation that hexes should be in
#[derive(Clone,Deserialize)]
pub enum Orientation {
    /// Hexes should have a flat top
    Horizontal,
    /// Hexes should have apoint at the top
    Vertical,
}

#[derive(Clone, Deserialize)]
pub struct Map {
    pub orientation: Orientation,
    pub scale: f64,
    pub width: u32,
    pub height: u32,
    #[serde(rename="tiles")]
    raw_tiles: Vec<MapTile>,
    #[serde(default)]
    pub barriers: Vec<Barrier>,
}

impl Default for Map {
    fn default() -> Map {
        Map {
            orientation: Orientation::Horizontal,
            scale: 3.81, // Hexes are usually 3.81cm flat-to-flat
            width: 5,
            height: 5,
            raw_tiles: vec![],
            barriers: vec![],
        }
    }
}

impl Map {
    pub fn load(dir: PathBuf,
                definitions: &HashMap<String, tile::TileDefinition>) -> Map {
        let mut map: Map;
        let map_filename = dir.join("map.json");
        if !dir.exists() {
            eprintln!("Can't find a game in {}", dir.to_string_lossy());
            process::exit(1);
        }

        println!("Reading map information...");
        match File::open(map_filename) {
            Err(e) => {
                eprintln!("Couldn't open map file: {}", e);
                process::exit(1);
            }
            Ok(file) => {
                map = serde_json::from_reader(file).unwrap_or_else(|err| {
                    eprintln!("Failed to parse map: {}", err);
                    process::exit(1);
                });
            }
        };
        // Connect the tiles to their definitions
        for tile in map.raw_tiles.iter_mut() {
            let base = tile.tile.clone();
            match definitions.get(&base) {
                Some(def) => tile.set_definition(def),
                None => {
                    eprintln!("Invalid tile for location {:?}: Unknown tile \
                              definition '{}'", &tile.location, &tile.tile);
                    process::exit(1);
                }
            }
        }
        map
    }

    pub fn tiles(&self) -> HashMap<(u32, u32), &tile::TileSpec> {
        self.raw_tiles.iter()
            .map(|t| (t.location, t as &tile::TileSpec))
            .collect()
    }
}

/// A collection of tile specificiations
pub struct Game {
    pub manifest: Manifest,
    pub map: Map,
    pub log: Option<Log>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            manifest: Manifest::default(),
            map: Map::default(),
            log: None,
        }
    }

    pub fn load(dir: PathBuf,
                definitions: &HashMap<String, tile::TileDefinition>) -> Game {
        let mut game = Game::new();
        let manifest_filename = dir.join("manifest.json");
        if !dir.exists() {
            eprintln!("Can't find a game in {}", dir.to_string_lossy());
            process::exit(1);
        }

        println!("Reading tile manifest...");
        match File::open(manifest_filename) {
            Err(e) => {
                eprintln!("Couldn't open manifest file: {}", e);
                process::exit(1);
            }
            Ok(file) => {
                game.manifest = serde_json::from_reader(file)
                    .unwrap_or_else(|err| {
                        eprintln!("Failed to parse manifest: {}", err);
                        process::exit(1);
                });
            }
        };
        // Connect the manifest to the tile definitions
        for tile in game.manifest.tiles.iter_mut() {
            let base = tile.base_tile();
            match definitions.get(&base) {
                Some(def) => tile.set_definition(def),
                None => {
                    eprintln!("Unknown base_tile '{}'", base);
                    process::exit(1);
                }
            }
        }

        // Load the map itself
        game.map = Map::load(dir, definitions);

        game
    }

    pub fn set_log(mut self, log: Log) -> Self {
        self.log = Some(log);
        self
    }

    pub fn placed_tiles(&self) -> HashMap<(u32, u32), PlacedTile> {
        let mut placed = HashMap::new();
        if let Some(ref log) = self.log {
            for action in log.log.iter() {
                if let &Action::TileLay { location, ref tile,
                                          ref orientation } = action {
                    let t = PlacedTile::new_from(self.manifest.tiles.iter()
                            .find(|t| t.name() == tile).unwrap())
                        .set_orientation(
                            tile::direction_to_angle(orientation));
                    placed.insert(location, t);
                }
            }
        }
        placed
    }
}

#[derive(Deserialize)]
pub struct Manifest {
    pub tiles: Vec<tile::Tile>,
    amounts: HashMap<String, u32>,
}

impl Default for Manifest {
    fn default() -> Manifest {
        Manifest {
            tiles: vec![],
            amounts: HashMap::new(),
        }
    }
}

impl Manifest {
    pub fn get_tile(&self, name: &String) -> Result<&tile::Tile, String> {
        for tile in &self.tiles {
            if tile.name() == name {
                return Ok(tile);
            }
        }
        Err(format!("Tile with name '{}' not found in manifest", name))
    }

    pub fn amounts(&self, log: &Option<Log>) -> HashMap<String, u32> {
        match log {
            // Don't bother with the amount if there is no log
            &None => self.amounts.clone(),
            &Some(ref log) => {
                let mut placed: HashMap<(u32, u32), &String> = HashMap::new();
                let mut used: HashMap<&String, u32> = HashMap::new();
                for action in log.log.iter() {
                    if let &Action::TileLay{ location, ref tile, ..} = action {
                        let old_tile = placed.insert(location, tile);
                        if let Some(old_tile) = old_tile {
                            *used.entry(old_tile).or_insert(0) -= 1;
                        }
                        *used.entry(tile).or_insert(0) += 1;
                    }
                }
                self.amounts.iter()
                    .map(|(k, v)| (k.clone(), v - *used.get(k).unwrap_or(&0)))
                    .collect()
            }

        }
    }
}

#[derive(Clone,Deserialize)]
pub struct MapTile {
    pub location: (u32, u32),
    #[serde(default="MapTile::default_tile")]
    pub tile: String,

    // Optional parameters
    color: Option<String>,
    orientation: Option<String>,
    #[serde(default)]
    text: HashMap<String, String>,
    arrows: Option<Vec<tile::Coordinate>>,
    revenue: Option<tile::RevenueTrack>,

    #[serde(skip)]
    definition: Option<tile::TileDefinition>,
}

impl MapTile {
    pub fn set_definition(&mut self, definition: &tile::TileDefinition) {
        self.definition = Some(definition.clone())
    }

    pub fn default_tile() -> String {
        String::from("plain")
    }
}

impl TileSpec for MapTile {
   fn color(&self) -> tile::colors::Color {
       match &self.color {
           &None => tile::colors::GROUND,
           &Some(ref c) => tile::colors::name_to_color(&c),
       }
   }

   fn set_name(&mut self, _name: String) { }

   fn name(&self) -> &str {
       ""
   }

   fn paths(&self) -> Vec<tile::Path> {
       self.definition.as_ref()
           .expect("You must call set_definition() before using paths()")
           .paths()
   }

   fn cities(&self) -> Vec<tile::City> {
       self.definition.as_ref()
           .expect("You must call set_definition() before using cities()")
           .cities()
   }

   fn stops(&self) -> Vec<tile::Stop> {
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
            None => "", // Undefined text should be invisible
        }
    }

   fn text_position(&self, id: usize) -> Option<na::Vector3<f64>> {
       self.definition.as_ref()
           .expect("You must call set_definition() before using \
                   text_position")
           .text_position(id)
   }

    fn text_spec(&self) -> Vec<tile::Text> {
        self.definition.as_ref()
            .expect("You must call set_definition() before using \
                    text_spec()")
            .text_spec()
    }

   fn orientation(&self) -> f64 {
       match &self.orientation {
           &None => 0.0,
           &Some(ref o) => tile::direction_to_angle(o),
       }
   }

   fn arrows(&self) -> Vec<tile::Coordinate> {
       match &self.arrows {
           &None => vec![],
           &Some(ref arrows) => arrows.to_vec(),
       }
   }

   fn revenue_track(&self) -> Option<tile::RevenueTrack> {
       self.revenue.clone()
   }
}

pub struct PlacedTile<'a> {
    base_tile: &'a tile::TileSpec,
    orientation: f64,
}

impl<'a> PlacedTile<'a> {
    pub fn new_from(tile: &'a tile::TileSpec) -> PlacedTile {
        PlacedTile {
            base_tile: tile,
            orientation: 0.0,
        }
    }

    pub fn set_orientation(mut self, orientation: f64) -> Self {
        self.orientation = orientation;
        self
    }
}

impl<'a> TileSpec for PlacedTile<'a> {
    fn color(&self) -> tile::colors::Color { self.base_tile.color() }
    fn set_name(&mut self, _name:String) {
        panic!("Cannot change name of a PlacedTile");
    }
    fn name(&self) -> &str { self.base_tile.name() }
    fn paths(&self) -> Vec<tile::Path> { self.base_tile.paths() }
    fn cities(&self) -> Vec<tile::City> { self.base_tile.cities() }
    fn stops(&self) -> Vec<tile::Stop> { self.base_tile.stops() }
    fn is_lawson(&self) -> bool { self.base_tile.is_lawson() }
    fn arrows(&self) -> Vec<tile::Coordinate> { self.base_tile.arrows() }
    fn revenue_track(&self) -> Option<tile::RevenueTrack> {
        self.base_tile.revenue_track()
    }
    fn get_text<'b>(&'b self, id: &'b str) -> &'b str {
        self.base_tile.get_text(id)
    }
    fn text_position(&self, id: usize) -> Option<na::Vector3<f64>> {
        self.base_tile.text_position(id)
    }
    fn text_spec(&self) -> Vec<tile::Text> { self.base_tile.text_spec() }
    fn orientation(&self) -> f64 {
        self.orientation
    }
}

#[derive(Clone,Deserialize)]
pub struct Barrier {
    pub location: (u32, u32),
    pub side: String,
}

#[derive(Deserialize, Serialize)]
pub struct Log {
    pub game_name: String,
    pub log: Box<[Action]>,
}

impl Log {
    pub fn new() -> Log {
        Log {
            game_name: "1830".to_string(),
            log: Box::new([]),
        }
    }

    pub fn new_game(game: String) -> Log {
        Log {
            game_name: game,
            log: Box::new([]),
        }
    }

    pub fn load(state_options: &super::StateOptions,
                _options: &super::Options) -> Log {
        let log: Log;
        let log_filename = format!("{}.json", state_options.name);
        println!("Reading log from file...");
        match File::open(log_filename) {
            Err(e) => {
                eprintln!("Failed to find game file: {}", e);
                process::exit(1);
            }
            Ok(file) => {
                log = serde_json::from_reader(file).unwrap_or_else(|err| {
                    eprintln!("Failed to load game: {}", err);
                    process::exit(1);
                });
            }
        }
        log
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all="lowercase", tag="type")]
pub enum Action {
    TileLay { location: (u32, u32), tile: String, orientation: String },
    Token { location: (u32, u32), company: String, city: Option<u32> },
}
