extern crate toml;

use std::collections::HashMap;
use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;

use tile;

/// Orientation that hexes should be in
#[derive(Deserialize)]
pub enum Orientation {
    /// Hexes should have a flat top
    Horizontal,
    /// Hexes should have apoint at the top
    Vertical,
}

#[derive(Deserialize)]
pub struct Map {
    pub orientation: Orientation,
    pub scale: f64,
}

impl Map {
    pub fn default () -> Map {
        Map {
            orientation: Orientation::Horizontal,
            scale: 3.81, // Hexes are usually 3.81cm flat-to-flat
        }
    }
}

/// A collection of tile specificiations
pub struct Game {
    pub manifest: Manifest,
    pub info: Map,
}

impl Game {
    pub fn new() -> Game {
        Game {
            manifest: Manifest::default(),
            info: Map::default(),
        }
    }

    pub fn set_directory(mut self, dir: PathBuf) -> Game {
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
                self.manifest = toml::from_str(&contents).unwrap();
            }
        };
        self
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
