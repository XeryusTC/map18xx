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
        let text_pos = helpers::scale(&info)
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
        let text_pos = helpers::scale(&game.map) *
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
        .add(helpers::draw_text(
                "Tile sheet 0",
                &(Vector2::new(2.0_f64, 0.5) * helpers::scale(&info)),
                &tile::TextAnchor::Start,
                Some("200%"), None));
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
                            &(Vector2::new(2.0, 0.5) * helpers::scale(&info)),
                            &tile::TextAnchor::Start,
                            Some("200%"), None));
            }
        }
    }
    sheets.push(cur_doc);
    sheets
}

/// Convert location to cube coordinate
fn convert_coord(col: i32, row: i32, game: &game::Game) -> [f64; 3] {
    match game.map.orientation {
        Orientation::Vertical => {
            let x = (col - (row - (row % 2)) / 2) as f64;
            let z = row as f64;
            [x, -x-z, -z]
        }
        Orientation::Horizontal => {
            let x = col as f64;
            let z = (row - (col - (col % 2)) / 2) as f64;
            [x, -x-z, -z]
        }
    }
}

/// Draw the map of a game
pub fn draw_map(game: &game::Game, options: &super::Options) -> svg::Document {
    let width: f64;
    let height: f64;
    let border_offset = 0.5;
    let offset: Vector2<f64>;
    match &game.map.orientation {
        &Orientation::Horizontal => {
            width = 0.3 * 3.0_f64.sqrt() + game.map.width as f64 * 1.5;
            height = (0.5 + game.map.height as f64) * 3.0_f64.sqrt();
            offset = Vector2::new(border_offset + 1.0,
                                  border_offset + 3.0_f64.sqrt() / 2.0);
        }
        &Orientation::Vertical => {
            width = (0.5 + game.map.width as f64) * 3.0_f64.sqrt();
            height = 0.3 * 3.0_f64.sqrt() + game.map.height as f64 * 1.5;
            offset = Vector2::new(border_offset + 3.0_f64.sqrt() / 2.0,
                                  border_offset + 1.0);
        }
    }
    let page_width = (width + 2.0 * border_offset) * helpers::scale(&game.map);
    let page_height = (height + 2.0 * border_offset)*helpers::scale(&game.map);
    let basis = helpers::get_basis(&game.map.orientation);
    let mut doc = svg::Document::new()
        .set("width", format!("{}", page_width))
        .set("height", format!("{}", page_height));

    // Draw tiles
    for tile in game.map.tiles.iter() {
        let (x, y) = tile.location;
        let pos = offset + basis
            * na::Vector3::from(convert_coord(x as i32, y as i32, game))
                .component_mul(&na::Vector3::new(2.0, 1.0, 1.0));
        doc = doc.add(draw_tile(tile, &pos, &game.map));
    }

    // Draw borders
    for barrier in game.map.barriers.iter() {
        let (x, y) = barrier.location;
        let pos = offset + basis
            * na::Vector3::from(convert_coord(x as i32, y as i32, game))
                .component_mul(&na::Vector3::new(2.0, 1.0, 1.0));
        doc = doc.add(helpers::draw_barrier(barrier, &pos, &game.map));
    }

    // Draw coordinate system
    let hoffset: f64;
    let voffset: f64 = border_offset + 1.0;
    let hstride: f64;
    let vstride: f64;
    let mut hnums: u32 = 1;
    let mut vnums: u32 = 1;
    match game.map.orientation {
        Orientation::Horizontal => {
            if !options.debug_coordinates {
                vnums = 2;
            }
            hoffset = border_offset + 1.0;
            hstride = 1.5;
            vstride = 3.0_f64.sqrt() / vnums as f64;
        }
        Orientation::Vertical => {
            if !options.debug_coordinates {
                hnums = 2;
            }
            hoffset = border_offset + 0.5 * 3.0_f64.sqrt();
            hstride = 3.0_f64.sqrt() / hnums as f64;
            vstride = 1.5;
        }
    }
    let mut border = element::Group::new()
        .add(element::Rectangle::new()
            .set("x", border_offset * helpers::scale(&game.map))
            .set("y", border_offset * helpers::scale(&game.map))
            .set("width", width * helpers::scale(&game.map))
            .set("height", height * helpers::scale(&game.map))
            .set("fill", "none")
            .set("stroke", "black")
            .set("stroke-width",
                 consts::LINE_WIDTH * helpers::scale(&game.map)));
    for x in 0..(game.map.width * hnums) {
        let text = if options.debug_coordinates {
            x.to_string()
        } else {
            (x + 1).to_string()
        };
        let x1 = Vector2::new(x as f64 * hstride + hoffset,
                              0.75 * border_offset)
            * helpers::scale(&game.map);
        let x2 = Vector2::new(x as f64 * hstride + hoffset,
                              1.75 * border_offset + height)
            * helpers::scale(&game.map);
        border = border
            .add(helpers::draw_text(&text, &x1, &tile::TextAnchor::Middle,
                                    Some("16pt"), Some(600)))
            .add(helpers::draw_text(&text, &x2, &tile::TextAnchor::Middle,
                                    Some("16pt"), Some(600)));
    }
    for y in 0..(game.map.height * vnums) {
        let text = if options.debug_coordinates {
            y.to_string()
        } else {
            (y + 1).to_string()
        };
        let y1 = Vector2::new(0.5 * border_offset,
                              y as f64 * vstride + voffset)
            * helpers::scale(&game.map);
        let y2 = Vector2::new(1.5 * border_offset + width,
                              y as f64 * vstride + voffset)
            * helpers::scale(&game.map);
        border = border
            .add(helpers::draw_text(&text, &y1, &tile::TextAnchor::Middle,
                                    Some("16pt"), Some(600)))
            .add(helpers::draw_text(&text, &y2, &tile::TextAnchor::Middle,
                                    Some("16pt"), Some(600)));
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
        let text_pos = helpers::scale(&map) *
            (rotation * basis * text.position() + pos);
        let mut t = helpers::draw_text(&tile.get_text(text.id), &text_pos,
                                       &text.anchor, text.size(), text.weight);
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
