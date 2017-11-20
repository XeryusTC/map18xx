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

#[derive(Deserialize, Debug)]
pub struct TileDefinition {
    path: Option<Vec<Path>>,
    city: Option<Vec<City>>,
}

impl TileDefinition {
    fn paths(self) -> Option<Vec<Path>> {
        self.path
    }

    fn cities(&self) -> &Option<Vec<City>> {
        &self.city
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

#[derive(Deserialize, Debug)]
pub struct City {
    circles: u32,
    revenue: u32,
    position: Option<String>,
    pos: Option<Box<[f64]>>,
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
