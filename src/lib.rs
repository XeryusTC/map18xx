extern crate argparse;
#[macro_use]
extern crate serde_derive;
extern crate svg;

use argparse::ArgumentParser;

pub mod draw;
pub mod game;
pub mod tile;

/// Place to store command line options
struct Options {
    mode: String,
}

impl Options {
    pub fn new() -> Options {
        Options {
            mode: String::from("definitions"),
        }
    }
}

pub fn run() {
    let mut options = Options::new();
    { // Limit scope of ArgumentParser borrow
        let mut parser = ArgumentParser::new();
        parser.set_description("18xx tile and map designer.");
        parser.add_option(&["-V", "--version"],
                          argparse::Print(env!("CARGO_PKG_VERSION")
                                          .to_string()),
                          "Show version");
        parser.refer(&mut options.mode)
            .add_argument("mode",
                          argparse::Store,
                          "Mode to use (default: definitions)");
        parser.parse_args_or_exit();
    }

    match options.mode.as_ref() {
        "d" | "def" | "definitions" => definitions(),
        "game" => game_mode(&String::from("1830"), &options),
        m => {
            println!("Unrecognized mode '{}', falling back to definitions", m);
            definitions()
        }
    }
}

fn definitions() {
    let definitions = tile::definitions();
    let document = svg::Document::new()
        .set("width", "210mm") // A4 width
        .set("height",
             format!("{}mm", (definitions.len() as f64/3.0).ceil()*32.0+3.0))
        .add(draw::draw_tile_definitions(&definitions));
    svg::save("definitions.svg", &document).unwrap();
}

fn game_mode(name: &String, _options: &Options) {
    println!("Processing game '{}'", name);
    let definitions = tile::definitions();
    let game = game::Game::load(["games", name.as_str()].iter().collect(),
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
        let filename = format!("{}-sheet-{}.svg", name, i);
        svg::save(filename, sheet).unwrap();
    }

    println!("Exporting map...");
    let map_render = draw::draw_map(&game);
    svg::save(format!("{}-map.svg", name), &map_render).unwrap()
}
