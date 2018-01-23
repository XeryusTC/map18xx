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

#[derive(Clone,Deserialize)]
pub struct Map {
    pub orientation: Orientation,
    pub scale: f64,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<MapTile>,
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
            tiles: vec![],
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
        for tile in map.tiles.iter_mut() {
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
}

/// A collection of tile specificiations
pub struct Game {
    pub manifest: Manifest,
    pub map: Map,
}

impl Game {
    pub fn new() -> Game {
        Game {
            manifest: Manifest::default(),
            map: Map::default(),
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
}

#[derive(Deserialize)]
pub struct Manifest {
    pub tiles: Vec<tile::Tile>,
    pub amounts: HashMap<String, u32>,
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
    text: Box<[String]>,
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

    fn get_text(&self, id: usize) -> String {
        if id == 0 {
            String::from(self.name())
        } else if id > self.text.len() {
            String::new()
        } else {
            self.text[id - 1].to_string()
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

#[derive(Clone,Deserialize)]
pub struct Barrier {
    pub location: (u32, u32),
    pub side: String,
}
