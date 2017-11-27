extern crate toml;

use std::fs::File;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process;

use tile;

/// A collection of tile specificiations
pub struct Game {
    manifest: Manifest,
}

impl Game {
    pub fn new() -> Game {
        Game {
            manifest: Manifest::default(),
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
    tiles: Vec<tile::Tile>,
}

impl Default for Manifest {
    fn default() -> Manifest {
        Manifest {
            tiles: vec![],
        }
    }
}
