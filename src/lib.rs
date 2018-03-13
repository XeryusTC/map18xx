#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;
extern crate svg;

use std::fs;
use std::fs::OpenOptions;
use std::process;

pub mod draw;
pub mod game;
pub mod tile;

/// Place to store command line options
pub struct Options {
    pub verbose: bool,
    pub debug_coordinates: bool,
}

impl Options {
    pub fn new() -> Options {
        Options {
            verbose: false,
            debug_coordinates: false,
        }
    }
}

pub struct AssetOptions {
    pub name: String,
}

impl AssetOptions {
    pub fn new() -> AssetOptions {
        AssetOptions {
            name: String::new(),
        }
    }
}

pub struct NewGameOptions {
    pub game: String,
    pub name: String,
    pub overwrite: bool,
}

impl NewGameOptions {
    pub fn new() -> NewGameOptions {
        NewGameOptions {
            game: String::new(),
            name: String::new(),
            overwrite: false,
        }
    }
}

pub struct StateOptions {
    pub name: String,
}

impl StateOptions {
    pub fn new() -> StateOptions {
        StateOptions {
            name: String::new(),
        }
    }
}

pub fn definitions(options: &Options) {
    let definitions = tile::definitions(options);
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm", (definitions.len() as f64/4.0).ceil()*42.0+0.0))
        .add(draw::draw_tile_definitions(&definitions));
    svg::save("definitions.svg", &document).unwrap_or_else(|err| {
        eprintln!("Failed to write definitions.svg: {:?}", err.kind());
        process::exit(1);
    });
}

pub fn asset_mode(options: &Options, asset_options: &AssetOptions) {
    println!("Processing game '{}'", asset_options.name);
    let definitions = tile::definitions(options);
    let game = game::Game::load(["games", asset_options.name.as_str()]
                                    .iter().collect(),
                                &definitions);

    println!("Exporting tile manifest...");
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm",
                     (game.manifest.tiles.len() as f64 / 4.0).ceil()
                     * (game.map.scale * 10.0 + 3.0)))
        .add(draw::draw_tile_manifest(&game));
    svg::save(format!("{}-manifest.svg", asset_options.name), &document)
        .unwrap_or_else(|err| {
            eprintln!("Failed to write {}-manifest.svg: {:?}",
                      asset_options.name, err.kind());
            process::exit(1);
    });

    println!("Exporting tile sheets...");
    let sheets = draw::draw_tile_sheets(&game);
    for (i, sheet) in sheets.iter().enumerate() {
        let filename = format!("{}-sheet-{}.svg", asset_options.name, i);
        svg::save(filename, sheet).unwrap_or_else(|err| {
            eprintln!("Failed to write {}-sheet-{}.svg: {:?}",
                      asset_options.name, i, err.kind());
            process::exit(1);
        });
    }

    println!("Exporting map...");
    let map_render = draw::draw_map(&game, &options);
    svg::save(format!("{}-map.svg", asset_options.name), &map_render)
        .unwrap_or_else(|err| {
            eprintln!("Failed to write {}-map.svg: {:?}", asset_options.name,
                      err.kind());
            process::exit(1);
    });
}

pub fn newgame_mode(_options: &Options, newgame_options: &NewGameOptions) {
    // Validate that the game exists
    let games: Vec<String> = match fs::read_dir("games") {
        Err(e) => {
            eprintln!("Couldn't open games directory: {}", e);
            process::exit(1);
        }
        Ok(paths) => {
            paths.map(|path| path.unwrap().file_name().into_string().unwrap())
                .collect()
        }
    };
    if !games.contains(&newgame_options.game) {
        eprintln!("Game '{}' does not exist", newgame_options.game);
        process::exit(1);
    }

    let log = game::Log::new_game(newgame_options.game.clone());
    let file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .create_new(!newgame_options.overwrite)
        .open(format!("{}.yaml", newgame_options.name));
    match file {
        Err(e) => {
            eprintln!("Couldn't create game file {}.yaml: {}",
                      newgame_options.name, e);
            process::exit(1);
        }
        Ok(file) => {
            println!("Writing to {}.yaml", newgame_options.name);
            serde_yaml::to_writer(file, &log).unwrap();
        }
    }
}

pub fn game_state_mode(options: &Options, state_options: &StateOptions) {
    let log = game::Log::load(state_options, options);
    let definitions = tile::definitions(options);
    let game = game::Game::load(
            ["games", log.game_name.as_str()].iter().collect(),
            &definitions)
        .set_log(log);

    println!("Exporting tile manifest...");
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm",
                     (game.manifest.tiles.len() as f64 / 3.0).ceil()*30.0+3.0))
        .add(draw::draw_tile_manifest(&game));
    svg::save(format!("{}-manifest.svg", state_options.name), &document)
        .unwrap_or_else(|err| {
            eprintln!("Failed to write {}-manifest.svg: {}",
                      state_options.name, err);
            process::exit(1);
        });

    println!("Exporting map...");
    let map_render = draw::draw_map(&game, &options);
    svg::save(format!("{}-map.svg", state_options.name), &map_render)
        .unwrap_or_else(|err| {
            eprintln!("Failed to write {}-map.svg: {}",
                      state_options.name, err);
            process::exit(1);
        });
}
