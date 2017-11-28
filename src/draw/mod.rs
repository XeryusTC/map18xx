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
use super::Orientation;

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

/// Draws sheets with tiles of them for printing
pub fn draw_tile_sheets(manifest: &game::Manifest,
                        info: &map::MapInfo) -> Vec<svg::Document> {
    const TILES_PER_PAGE: u32 = 30;
    const TILES_PER_COL: u32 = 6;
    // Always draw vertical (fits more on a page)
    let info = map::MapInfo {
        orientation: super::Orientation::Vertical,
        ..*info
    };
    let mut drawn = 0;
    let mut sheets = vec![];
    let mut cur_doc = svg::Document::new()
        .set("width", "210mm")
        .set("height", "297mm")
        .add(helpers::draw_text(&"Tile sheet 0".to_string(),
                                &(Vector2::new(2.0_f64, 0.5) * info.scale *
                                  consts::PPCM),
                                helpers::TextAnchor::Start, Some("200%"),
                                None));
    for tile in manifest.tiles.iter() {
        for _ in 0..*manifest.amounts.get(tile.name()).unwrap() {
            let x = ((drawn % TILES_PER_PAGE) / TILES_PER_COL) as f64;
            let y = (drawn % TILES_PER_COL) as f64;
            let pos = Vector2::new(3.0_f64.sqrt() * (x + 1.0),
                                   2.0 * y + 1.75 + (x % 2.0));
            cur_doc = cur_doc.add(draw_tile(tile, &pos, &info));
            drawn += 1;
            // When a sheet is full, append this to the list and start a
            // new page
            if drawn % TILES_PER_PAGE == 0 {
                sheets.push(cur_doc);
                cur_doc = svg::Document::new()
                    .set("width", "210mm")
                    .set("height", "297mm")
                    .add(helpers::draw_text(
                            &format!("Tile sheet {}", drawn/TILES_PER_PAGE),
                            &(Vector2::new(2.0_f64, 0.5) * info.scale *
                              consts::PPCM),
                            helpers::TextAnchor::Start, Some("200%"), None));
            }
        }
    }
    sheets.push(cur_doc);
    sheets
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
    let mut text = helpers::draw_text(&String::from(tile.name()), &text_pos,
                                      helpers::TextAnchor::End, Some("80%"),
                                      None);
    if let Orientation::Vertical = map.orientation {
        text = text.set("transform",
                        format!("rotate(-30 {} {})", text_pos.x, text_pos.y));
    }
    g = g.add(text);

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
