extern crate toml;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
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
}

impl Default for Map {
    fn default() -> Map {
        Map {
            orientation: Orientation::Horizontal,
            scale: 3.81, // Hexes are usually 3.81cm flat-to-flat
            width: 5,
            height: 5,
            tiles: vec![],
        }
    }
}

impl Map {
    pub fn load(dir: PathBuf) -> Map {
        let map: Map;
        let map_filename = dir.join("map.toml");
        if !dir.exists() {
            eprintln!("Can't find a game in {}", dir.to_string_lossy());
            process::exit(1);
        }

        println!("Reading map information...");
        let mut contents = String::new();
        match File::open(map_filename) {
            Err(e) => {
                eprintln!("Couldn't open map file: {}", e);
                process::exit(1);
            }
            Ok(mut file) => {
                file.read_to_string(&mut contents).unwrap();
                map = toml::from_str(&contents).unwrap();
            }
        };
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
        let manifest_filename = dir.join("manifest.toml");
        if !dir.exists() {
            eprintln!("Can't find a game in {}", dir.to_string_lossy());
            process::exit(1);
        }

        println!("Reading tile manifest...");
        let mut contents = String::new();
        match File::open(manifest_filename) {
            Err(e) => {
                eprintln!("Couldn't open manifest file: {}", e);
                process::exit(1);
            }
            Ok(mut file) => {
                file.read_to_string(&mut contents).unwrap();
                game.manifest = toml::from_str(&contents).unwrap();
            }
        };
        // Connect the manifest to the tile definitions
        for tile in game.manifest.tiles.iter_mut() {
            let base = tile.base_tile();
            tile.set_definition(definitions.get(&base).unwrap());
        }

        // Load the map itself
        game.map = Map::load(dir);

        game
    }
}

#[derive(Deserialize)]
pub struct Manifest {
    #[serde(rename="tile")]
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
    pub tile: String,
}
