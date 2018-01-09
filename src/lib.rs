extern crate argparse;
#[macro_use]
extern crate serde_derive;
extern crate svg;

use argparse::ArgumentParser;
use std::io::{stdout, stderr};
use std::process;

pub mod draw;
pub mod game;
pub mod tile;

/// Place to store command line options
pub struct Options {
    mode: String,
    verbose: bool,
}

impl Options {
    pub fn new() -> Options {
        Options {
            mode: String::from("definitions"),
            verbose: false,
        }
    }
}

struct GameOptions {
    name: String,
}

impl GameOptions {
    pub fn new() -> GameOptions {
        GameOptions {
            name: String::new(),
        }
    }
}

pub fn run() {
    let mut options = Options::new();
    let mut args = vec![];
    { // Limit scope of ArgumentParser borrow
        let mut parser = ArgumentParser::new();
        parser.set_description("18xx tile and map designer.");
        parser.add_option(&["-V", "--version"],
                          argparse::Print(env!("CARGO_PKG_VERSION")
                                          .to_string()),
                          "Show version");
        parser.refer(&mut options.verbose)
            .add_option(&["-v", "--verbose"],
                        argparse::StoreTrue,
                        "Print debug information");
        parser.refer(&mut options.mode)
            .add_argument("mode",
                          argparse::Store,
                          "Mode to use (default: definitions)");
        parser.refer(&mut args)
            .add_argument("args",
                          argparse::List,
                          "Arguments for mode");
        parser.stop_on_first_argument(true);
        parser.parse_args_or_exit();
    }

    match options.mode.as_ref() {
        "d" | "def" | "definitions" => definitions(&options),
        "game" => {
            args.insert(0, String::from("game"));
            game_mode(&options, args)
        }
        m => {
            println!("Unrecognized mode '{}'. See 'map18xx --help'", m);
            process::exit(1);
        }
    }
}

fn definitions(options: &Options) {
    let definitions = tile::definitions(options);
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm", (definitions.len() as f64/4.0).ceil()*42.0+0.0))
        .add(draw::draw_tile_definitions(&definitions));
    svg::save("definitions.svg", &document).unwrap();
}

fn game_mode(options: &Options, args: Vec<String>) {
    let mut game_options = GameOptions::new();
    { // Limit scope of ArgumentParser borrow
        let mut parser = ArgumentParser::new();
        parser.set_description("Game mode");
        parser.refer(&mut game_options.name).required()
            .add_argument("name",
                          argparse::Store,
                          "Game for which to generate files");
        match parser.parse(args, &mut stdout(), &mut stderr()) {
            Ok(()) => {}
            Err(x) => process::exit(x),
        }
    }

    println!("Processing game '{}'", game_options.name);
    let definitions = tile::definitions(options);
    let game = game::Game::load(["games", game_options.name.as_str()]
                                    .iter().collect(),
                                &definitions);

    println!("Exporting tile manifest...");
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm",
                     (game.manifest.tiles.len() as f64/3.0).ceil()*30.0+3.0))
        .add(draw::draw_tile_manifest(&game));
    svg::save("manifest.svg", &document).unwrap();

    println!("Exporting tile sheets...");
    let sheets = draw::draw_tile_sheets(&game);
    for (i, sheet) in sheets.iter().enumerate() {
        let filename = format!("{}-sheet-{}.svg", game_options.name, i);
        svg::save(filename, sheet).unwrap();
    }

    println!("Exporting map...");
    let map_render = draw::draw_map(&game);
    svg::save(format!("{}-map.svg", game_options.name), &map_render).unwrap()
}
