extern crate svg;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::process;
use self::svg::node::element::Group;
use self::svg::node::element;
use tile;
use tile::TileSpec;
use game;
use self::na::Vector2;
use game::Orientation;

mod helpers;
mod consts;

const TILES_PER_ROW: f64 = 4.0;

/// Draws tile definitions
pub fn draw_tile_definitions(
        definitions: &HashMap<String, tile::TileDefinition>) -> Group {
    println!("Drawing tile definitions...");
    let mut g = Group::new();
    let mut i = 0.0;
    let info = game::Map::default();

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
                                    &tile::TextAnchor::Start, None, None));
        i += 1.0;
    }
    g
}

/// Draw a game's tile manifest
pub fn draw_tile_manifest(game: &game::Game) -> Group {
    let mut g = Group::new();
    let mut i = 0.0;

    for tile in &game.manifest.tiles {
        let pos = Vector2::new(1.1_f64 + 2.25 * (i % TILES_PER_ROW),
                                   1.0 + 2.0 * (i / TILES_PER_ROW).floor());
        g = g.add(draw_tile(tile, &pos, &game.map));
        i += 1.0;

        // Draw amount available
        let amount = match game.manifest.amounts.get(tile.name()) {
            None => {
                eprintln!("No tile amount found for {}", tile.name());
                process::exit(1);
            }
            Some(amount) => amount.to_string(),
        };
        let text_pos = consts::PPCM * game.map.scale *
            Vector2::new(pos.x-1.0, pos.y-0.7);
        g = g.add(helpers::draw_text(&format!("{}×", amount), &text_pos,
                                     &tile::TextAnchor::Start, None, None));
    }

    g
}

/// Draws sheets with tiles of them for printing
pub fn draw_tile_sheets(game: &game::Game) -> Vec<svg::Document> {
    const TILES_PER_PAGE: u32 = 30;
    const TILES_PER_COL: u32 = 6;
    // Always draw vertical (fits more on a page)
    let info = game::Map {
        orientation: Orientation::Vertical,
        ..game.map.clone()
    };
    let mut drawn = 0;
    let mut sheets = vec![];
    let mut cur_doc = svg::Document::new()
        .set("width", "210mm")
        .set("height", "297mm")
        .add(helpers::draw_text(&"Tile sheet 0".to_string(),
                                &(Vector2::new(2.0_f64, 0.5) * info.scale *
                                  consts::PPCM),
                                &tile::TextAnchor::Start,
                                Some(String::from("200%")), None));
    for tile in game.manifest.tiles.iter() {
        for _ in 0..*game.manifest.amounts.get(tile.name()).unwrap() {
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
                            &tile::TextAnchor::Start,
                            Some(String::from("200%")), None));
            }
        }
    }
    sheets.push(cur_doc);
    sheets
}

/// Draw the map of a game
pub fn draw_map(game: &game::Game, options: &super::Options) -> svg::Document {
    let hor_dist: f64;
    let ver_dist: f64;
    let hor_offset: f64;
    let ver_offset: f64;
    let row_offset: f64;
    let col_offset: f64;
    let width: f64;
    let height: f64;
    let mut hor_nums: u32 = 1;
    let mut ver_nums: u32 = 1;
    let border_offset = 0.5;
    let border = na::Vector2::new(border_offset, border_offset);
    match &game.map.orientation {
        &Orientation::Horizontal => {
            hor_dist = 1.5;
            ver_dist = 3.0_f64.sqrt();
            hor_offset = 2.0 / 3.0;
            ver_offset = 0.5;
            row_offset = 0.0;
            col_offset = 0.5 * 3.0_f64.sqrt();
            width = 0.3 * game.map.scale + game.map.width as f64 *
                (0.5 * game.map.scale * 3.0_f64.sqrt());
            height = (0.5 + game.map.height as f64) * game.map.scale;
            hor_nums = 1;
            if options.pretty_coordinates {
                ver_nums = 2;
            }
        }
        &Orientation::Vertical => {
            hor_dist = 3.0_f64.sqrt();
            ver_dist = 1.5;
            hor_offset = 0.5;
            ver_offset = 2.0 / 3.0;
            row_offset = hor_dist * 0.5;
            col_offset = 0.0;
            width = (0.5 + game.map.width as f64) * game.map.scale;
            height = 0.3 * game.map.scale + game.map.height as f64 *
                (game.map.scale * 0.5 * 3.0_f64.sqrt());
            if options.pretty_coordinates {
                hor_nums = 2;
            }
            ver_nums = 1;
        }
    }
    let page_width = (width * 3.0_f64.sqrt() + 2.0 * border_offset *
                      game.map.scale) * consts::PPCM;
    let page_height = (height * 3.0_f64.sqrt() + 2.0 * border_offset *
                       game.map.scale) * consts::PPCM;
    let mut doc = svg::Document::new()
        .set("width", format!("{}", page_width))
        .set("height", format!("{}", page_height));

    // Draw tiles
    for tile in game.map.tiles.iter() {
        let (x, y) = tile.location;
        let pos = Vector2::new(
            (x as f64 + hor_offset) * hor_dist + (y % 2) as f64 * row_offset,
            (y as f64 + ver_offset) * ver_dist + (x % 2) as f64 * col_offset)
            + border;
        doc = doc.add(draw_tile(tile, &pos, &game.map));
    }

    // Draw borders
    for barrier in game.map.barriers.iter() {
        let (x, y) = barrier.location;
        let pos = na::Vector2::new(
            (x as f64 + hor_offset) * hor_dist + (y % 2) as f64 * row_offset,
            (y as f64 + ver_offset) * ver_dist + (x % 2) as f64 * col_offset)
            + border;
        doc = doc.add(helpers::draw_barrier(barrier, &pos, &game.map));
    }

    // Draw coordinate system
    let mut border = element::Group::new()
        .add(element::Rectangle::new()
            .set("x", border_offset * consts::PPCM * game.map.scale)
            .set("y", border_offset * consts::PPCM * game.map.scale)
            .set("width", width * consts:: PPCM * 3.0_f64.sqrt())
            .set("height", height * consts::PPCM * 3.0_f64.sqrt())
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width",
                 consts::LINE_WIDTH * consts::PPCM * game.map.scale));
    for x in 0..(game.map.width * hor_nums) {
        let x_off = border_offset + hor_offset * hor_dist;
        let pos = na::Vector2::new(
            x as f64 * hor_dist / hor_nums as f64 + x_off,
            0.75 * border_offset) * consts::PPCM * game.map.scale;
        border = border.add(helpers::draw_text(&(x + 1).to_string(),
                                               &pos,
                                               &tile::TextAnchor::Middle,
                                               Some(String::from("16pt")),
                                               Some(600)));
        let pos = pos + na::Vector2::new(0.0, (height * 3.0_f64.sqrt() +
                            border_offset * game.map.scale) * consts::PPCM);
        border = border.add(helpers::draw_text(&(x + 1).to_string(),
                                               &pos,
                                               &tile::TextAnchor::Middle,
                                               Some(String::from("16pt")),
                                               Some(600)));
    }
    for y in 0..(game.map.height * ver_nums) {
        let y_off = border_offset + ver_offset * ver_dist;
        let pos = na::Vector2::new(
            0.5 * border_offset,
            y as f64 * ver_dist / ver_nums as f64 + y_off)
            * consts::PPCM * game.map.scale;
        border = border.add(helpers::draw_text(&(y + 1).to_string(),
                                               &pos,
                                               &tile::TextAnchor::Middle,
                                               Some(String::from("16pt")),
                                               Some(600)));
        let pos = pos + na::Vector2::new(
            (width * 3.0_f64.sqrt() + border_offset * game.map.scale) *
                consts::PPCM,
            0.0);
        border = border.add(helpers::draw_text(&(y + 1).to_string(),
                                               &pos,
                                               &tile::TextAnchor::Middle,
                                               Some(String::from("16pt")),
                                               Some(600)));
    }

    doc.add(border)
}

/// Draws a single tile
pub fn draw_tile<T>(tile: &T,
                 pos: &Vector2<f64>,
                 map: &game::Map) -> Group
        where T: tile::TileSpec
{
    let mut g = Group::new();
    let basis = helpers::get_basis(&map.orientation);
    let rotation = helpers::rotate(&tile.orientation());

    g = g.add(helpers::draw_hex_background(*pos, &map, tile.color()));

    // Draw white contrast lines first
    for path in tile.paths() {
        g = g.add(helpers::draw_path_contrast(&path, pos, &map,
                                              &tile.orientation()));
    }
    for city in tile.cities() {
        g = g.add(helpers::draw_city_contrast(city, pos, &map,
                                              &tile.orientation()));
    };

    // Draw elements
    if tile.is_lawson() {
        g = g.add(helpers::draw_lawson(*pos, &map));
    }
    for path in tile.paths() {
        g = g.add(helpers::draw_path(&path, pos, &map, &tile.orientation()));
    };

    for stop in tile.stops() {
        g = g.add(helpers::draw_stop(stop, *pos, &map, tile,
                                     &tile.orientation()));
    }

    for city in tile.cities() {
        g = g.add(helpers::draw_city(city, *pos, &map, tile,
                                     &tile.orientation()));
    }

    for arrow in tile.arrows() {
        g = g.add(helpers::draw_arrow(&arrow, pos, &map));
    }

    // Draw text on tile
    for text in tile.text_spec() {
        let text_pos = consts::PPCM * map.scale *
            (rotation * basis * text.position() + pos);
        let mut t = helpers::draw_text(&tile.get_text(text.id), &text_pos,
                                       &text.anchor, text.size, text.weight);
        // Rotate the tile number with the orientation of the map
        if let Orientation::Vertical = map.orientation {
            t = t.set("transform",
                      format!("rotate(-30 {} {})", text_pos.x, text_pos.y));
        }
        g = g.add(t);
    }

    // Draw revenue track
    if let Some(track) = tile.revenue_track() {
        g = g.add(helpers::draw_revenue_track(&track, pos, &map));
    }

    // Draw outline last to prevent visual effects
    g.add(helpers::draw_hex_edge(*pos, &map))
}
