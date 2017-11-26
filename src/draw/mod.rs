extern crate svg;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::process;
use self::svg::node;
use self::svg::node::element::{Group, Text};
use tile;
use tile::TileSpec;
use map;

mod helpers;
mod consts;

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
        let pos = na::Vector2::new(1.1_f64 + 2.25 * (i % 5.0),
                                   1.0 + 2.0 * (i / 5.0).floor());
        g = g.add(draw_tile(&definition, &pos, &info))
            .add(Text::new()
                .add(node::Text::new(name.as_str()))
                .set("x", info.scale * (pos.x - 1.0))
                .set("y", info.scale * (pos.y - 0.7)));
        i += 1.0;
    }
    g
}

/// Draws a single tile
pub fn draw_tile(tile: &tile::TileDefinition,
                 pos: &na::Vector2<f64>,
                 map: &map::MapInfo) -> Group {
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
        g = g.add(helpers::draw_stop(stop, *pos, &map));
    }

    for city in tile.cities() {
        g = g.add(helpers::draw_city(city, *pos, &map));
    };

    // Draw tile number
    let text_pos = map.scale*(basis * na::Vector3::new(0.0, 0.0, -0.95) + pos);
    g = g.add(Text::new()
          .add(node::Text::new(tile.name()))
          .set("x", text_pos.x)
          .set("y", text_pos.y)
          .set("style", "text-anchor:end;font-size:80%"));
    // Draw the tile code
    match tile.code_text_id {
        Some(text_id) => {
            match tile.code_position() {
                None => {
                    eprintln!("Tile {} must have code_position and {}",
                              tile.name(),
                              "code_text_id set at the same time");
                    process::exit(1);
                },
                Some(ref position) => {
                    let text_pos = map.scale * (basis * position + pos);
                    g = g.add(Text::new()
                              .add(node::Text::new(text_id.to_string()))
                              .set("x", text_pos.x)
                              .set("y", text_pos.y)
                              .set("style",
                                   format!("{};{};{}",
                                           "text-anchor:middle",
                                           "font-size:120%",
                                           "font-weight:900")));
                }
            }
        }
        None => {}
    }
    // Draw outline last to prevent visual effects
    g.add(helpers::draw_hex_edge(*pos, &map))
}
