extern crate nalgebra as na;
extern crate serde_yaml;

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

/// Location of a tile, token or other object on a map
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Location {
    Coord(u32, u32),
    Named(String),
}

impl Location {
    pub fn as_coord(&self, orientation: &Orientation) -> (u32, u32) {
        match *self {
            Location::Coord(x, y) => (x, y),
            Location::Named(ref s) => {
                // Convert letter(s) to number
                let a = s.chars().take_while(|c| c.is_alphabetic())
                    .map(|c| c.to_digit(36).unwrap() - 9)
                    .fold(0, |acc, d| 26 * acc + d) - 1;
                // Map 1 based counting in string to 0 based
                let b = (s.chars().skip_while(|c| c.is_alphabetic())
                    .collect::<String>().parse::<u32>().unwrap() - 1) / 2;
                match *orientation {
                    Orientation::Horizontal => (a, b),
                    Orientation::Vertical => (b, a),
                }
            }
        }
    }
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
        let map_filename = dir.join("map.yaml");
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
                map = serde_yaml::from_reader(file).unwrap_or_else(|err| {
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
            .map(|t| (t.location.as_coord(&self.orientation),
                      t as &tile::TileSpec))
            .collect()
    }
}

/// A collection of tile specificiations
pub struct Game {
    pub manifest: Manifest,
    pub map: Map,
    pub log: Option<Log>,
    pub companies: HashMap<String, Company>,
}

impl Game {
    pub fn new() -> Game {
        Game {
            manifest: Manifest::default(),
            map: Map::default(),
            log: None,
            companies: HashMap::new(),
        }
    }

    pub fn load(dir: PathBuf,
                definitions: &HashMap<String, tile::TileDefinition>) -> Game {
        let mut game = Game::new();
        let manifest_filename = dir.join("manifest.yaml");
        let companies_filename = dir.join("companies.yaml");
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
                game.manifest = serde_yaml::from_reader(file)
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

        println!("Reading companies...");
        match File::open(companies_filename) {
            Err(e) => {
                eprintln!("Couldn't open company definitions: {}", e);
                process::exit(1);
            }
            Ok(file) => {
                game.companies = serde_yaml::from_reader(file)
                    .unwrap_or_else(|err| {
                        eprintln!("Failed to parse companies: {}", err);
                        process::exit(1);
                    });
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
                if let &Action::TileLay {ref location, ref tile,
                                         ref orientation} = action {
                    let location = location.as_coord(&self.map.orientation);
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

    pub fn tokens(&self) -> HashMap<(u32, u32), Vec<Token>> {
        let mut tokens = HashMap::new();
        for (name, company) in self.companies.iter() {
            if let Some(ref home) = company.home {
                let token = Token::from(&home, name, &company.color,
                                        &self.map.orientation).set_home();
                tokens.entry(token.location).or_insert(vec![]).push(token);
            }
        }
        // Update with placed tokens
        if let Some(ref log) = self.log {
            'next_action: for act in log.log.iter() {
                if let &Action::Token {ref location, ref company, city} = act {
                    let location = location.as_coord(&self.map.orientation);
                    let entry = tokens.entry(location).or_insert(vec![]);
                    let city = match city {
                        Some(id) => id,
                        None => 0,
                    } as usize;
                    let mut placed = 0;
                    for token in entry.iter_mut() {
                        if token.name == company.as_str() && token.is_home {
                            token.is_home = false;
                            continue 'next_action
                        }
                        if token.station == city {
                            placed += 1;
                        }
                    }
                    if placed >= top_tiles(&self.placed_tiles(),
                                           &self.map.tiles())
                                     .get(&location).unwrap()
                                     .cities().get(city).unwrap().circles
                    {
                        eprintln!("Could not place token for {} in {:?}: \
                                  too many tokens in station", company,
                                  location);
                    } else {
                        entry.push(
                            Token::place(self.companies.get(company).unwrap(),
                                         company, location, city, placed));
                    }
                }
                else if let &Action::RemoveCompany {ref company} = act {
                    for (_location, entry) in tokens.iter_mut() {
                        // Remove company
                        entry.retain(|t| t.name != *company);
                        entry.sort_by(|a, b| a.station.cmp(&b.station));

                        // Reorder remaining tokens
                        let mut station = entry[0].station;
                        let mut placed = 0;
                        for token in entry.iter_mut() {
                            if token.station != station {
                                station = token.station;
                                placed = 0;
                            }
                            token.circle = placed;
                            placed += 1;
                        }
                    }
                }
            }
        }

        tokens
    }
}

pub fn top_tiles<'a>(placed: &'a HashMap<(u32, u32), PlacedTile>,
                     tiles: &'a HashMap<(u32, u32), &tile::TileSpec>)
    -> HashMap<(u32, u32), &'a tile::TileSpec>
{
    let mut map: HashMap<(u32, u32), &tile::TileSpec> = HashMap::new();
    map.extend(tiles);
    placed.iter().map(|(k, t)| map.insert(*k, t as &tile::TileSpec)).count();
    map
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
                    if let &Action::TileLay{ref location, ref tile,
                                            ..} = action {
                        // For calculating manifest amounts we don't care about
                        // the exact rendered position, just the coordinate so
                        // we can use any orientation
                        let location = location
                            .as_coord(&Orientation::Horizontal);
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
    pub location: Location,
    #[serde(default="MapTile::default_tile")]
    pub tile: String,

    // Optional parameters
    color: Option<String>,
    orientation: Option<String>,
    #[serde(default)]
    text: HashMap<String, String>,
    arrows: Option<Vec<tile::Coordinate>>,
    revenue: Option<tile::RevenueTrack>,
    terrain: Option<tile::Terrain>,

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

   fn terrain(&self) -> Option<tile::Terrain> {
       self.terrain.clone()
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
    pub location: Location,
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
        let log_filename = format!("{}.yaml", state_options.name);
        println!("Reading log from file...");
        match File::open(log_filename) {
            Err(e) => {
                eprintln!("Failed to find game file: {}", e);
                process::exit(1);
            }
            Ok(file) => {
                log = serde_yaml::from_reader(file).unwrap_or_else(|err| {
                    eprintln!("Failed to load game: {}", err);
                    process::exit(1);
                });
            }
        }
        log
    }
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all="lowercase", tag="type")]
pub enum Action {
    TileLay { location: Location, tile: String, orientation: String },
    Token { location: Location, company: String, city: Option<u32> },
    RemoveCompany { company: String },
}

#[derive(Clone,Deserialize)]
#[serde(untagged)]
pub enum Home {
    PositionOnly(Location),
    PositionStation(Location, usize),
}

#[derive(Clone, Deserialize)]
pub struct Company {
    pub name: String,
    pub color: String,
    pub home: Option<Home>,
}

pub struct Token {
    pub name: String,
    pub color: String,
    pub location: (u32, u32),
    pub station: usize,
    pub circle: u32,
    pub is_home: bool,
}

impl Token {
    pub fn from(home: &Home, name: &str,
                color: &str,
                orientation: &Orientation) -> Self {
        let mut token = Token {
            name: name.to_string(),
            color: color.to_string(),
            location: (0, 0),
            station: 0,
            circle: 0,
            is_home: false,
        };
        match home {
            &Home::PositionOnly(ref loc) =>
                token.location = loc.as_coord(orientation),
            &Home::PositionStation(ref loc, s) => {
                token.location = loc.as_coord(orientation);
                token.station = s;
            }
        }
        token
    }

    pub fn place(company: &Company,
                 name: &str,
                 location: (u32, u32),
                 station: usize,
                 circle: u32) -> Self {
        Token {
            name: name.to_string(),
            color: company.color.to_string(),
            location,
            station,
            circle,
            is_home: false,
        }
    }

    pub fn set_home(mut self) -> Self {
        self.is_home = true;
        self
    }
}
