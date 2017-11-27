extern crate svg;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::process;
use self::svg::node::element::Group;
use tile;
use tile::TileSpec;
use map;
use game;
use self::na::{Vector2, Vector3};

mod helpers;
mod consts;

const TILES_PER_ROW: f64 = 4.0;

/// Draws tile definitions
pub fn draw_tile_definitions(
        definitions: &HashMap<String, tile::TileDefinition>) -> Group {
    println!("Drawing tile definitions...");
    let mut g = Group::new();
    let mut i = 0.0;
    let info = map::MapInfo::default();

    let mut keys: Vec<_> = definitions.keys().collect();
    keys.sort_by(|a, b| a.len().cmp(&b.len()).then(a.cmp(b)));
    for name in keys {
        let definition = &definitions[name];
        let pos = Vector2::new(1.1_f64 + 2.25 * (i % TILES_PER_ROW),
                                   1.0 + 2.0 * (i / TILES_PER_ROW).floor());
        let text_pos = consts::PPCM * info.scale
            * Vector2::new(pos.x - 1.0, pos.y - 0.7);
        g = g.add(draw_tile(definition, &pos, &info))
            .add(helpers::draw_text(&name, &text_pos,
                                    helpers::TextAnchor::Start, None, None));
        i += 1.0;
    }
    g
}

/// Draw a game's tile manifest
pub fn draw_tile_manifest(manifest: &game::Manifest,
                          info: &map::MapInfo) -> Group {
    let mut g = Group::new();
    let mut i = 0.0;

    for tile in &manifest.tiles {
        let pos = Vector2::new(1.1_f64 + 2.25 * (i % TILES_PER_ROW),
                                   1.0 + 2.0 * (i / TILES_PER_ROW).floor());
        g = g.add(draw_tile(tile, &pos, info));
        i += 1.0;

        // Draw amount available
        let amount = match manifest.amounts.get(tile.name()) {
            None => {
                eprintln!("No tile amount found for {}", tile.name());
                process::exit(1);
            }
            Some(amount) => amount.to_string(),
        };
        let text_pos = consts::PPCM * info.scale *
            Vector2::new(pos.x-1.0, pos.y-0.7);
        g = g.add(helpers::draw_text(&format!("{}×", amount), &text_pos,
                                     helpers::TextAnchor::Start, None, None));
    }

    g
}

/// Draws a single tile
pub fn draw_tile<T>(tile: &T,
                 pos: &Vector2<f64>,
                 map: &map::MapInfo) -> Group
        where T: tile::TileSpec
{
    let mut g = Group::new();
    let basis = helpers::get_basis(&map.orientation);

    g = g.add(helpers::draw_hex_background(*pos, &map, tile.color()));

    // Draw white contrast lines first
    for path in tile.paths() {
        g = g.add(helpers::draw_path_contrast(&path, pos, &map));
    }
    for city in tile.cities() {
        g = g.add(helpers::draw_city_contrast(city, pos, &map));
    };

    // Draw elements
    if tile.is_lawson() {
        g = g.add(helpers::draw_lawson(*pos, &map));
    }
    for path in tile.paths() {
        g = g.add(helpers::draw_path(&path, pos, &map));
    };

    for stop in tile.stops() {
        g = g.add(helpers::draw_stop(stop, *pos, &map, tile));
    }

    for city in tile.cities() {
        g = g.add(helpers::draw_city(city, *pos, &map, tile));
    };

    // Draw tile number
    let text_pos = consts::PPCM * map.scale *
        (basis * Vector3::new(0.0, 0.0, -0.95) + pos);
    g = g.add(helpers::draw_text(&String::from(tile.name()), &text_pos,
                                 helpers::TextAnchor::End, Some("80%"), None));
    // Draw the tile code
    match tile.code_text() {
        None => {}
        Some(text) => {
            match tile.code_position() {
                None => {
                    eprintln!("Tile {} must have code_position and {}",
                              tile.name(),
                              "code_text_id set at the same time");
                    process::exit(1);
                },
                Some(ref position) => {
                    let text_pos = consts::PPCM * map.scale *
                        (basis * position + pos);
                    g = g.add(helpers::draw_text(&text, &text_pos,
                                                 helpers::TextAnchor::Middle,
                                                 Some("120%"), Some(900)));
                }
            }
        }
    }
    // Draw outline last to prevent visual effects
    g.add(helpers::draw_hex_edge(*pos, &map))
}
