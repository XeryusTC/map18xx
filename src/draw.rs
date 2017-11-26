extern crate svg;
extern crate nalgebra as na;

use std::collections::HashMap;
use std::f64::consts::PI;
use std::process;
use self::svg::node;
use self::svg::node::element::{Circle, Group, Path, Rectangle, Text};
use self::svg::node::element::path::Data;
use super::Orientation;
use super::tile;
use super::tile::TileSpec;
use super::map;

// Bezier constant taken from
// http://spencermortensen.com/articles/bezier-circle/
const C: f64 = 0.551915024494;
const PATH_WIDTH: f64 = 0.1;
const LINE_WIDTH: f64 = 0.015;
const TOKEN_SIZE: f64 = 0.269;
const STOP_SIZE:  f64 = 0.1;
const STOP_TEXT_DIST: f64 = 0.4;
const REVENUE_CIRCLE_RADIUS: f64 = 0.13;

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
    let basis = get_basis(&map.orientation);

    g = g.add(draw_hex_background(*pos, &map, tile.color()));

    // Draw white contrast lines first
    for path in tile.paths() {
        g = g.add(draw_path_contrast(&path, pos, &map));
    }
    for city in tile.cities() {
        g = g.add(draw_city_contrast(city, pos, &map));
    };

    // Draw elements
    if tile.is_lawson() {
        g = g.add(draw_lawson(*pos, &map));
    }
    for path in tile.paths() {
        g = g.add(draw_path(&path, pos, &map));
    };

    for stop in tile.stops() {
        g = g.add(draw_stop(stop, *pos, &map));
    }

    for city in tile.cities() {
        g = g.add(draw_city(city, *pos, &map));
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
    g.add(draw_hex_edge(*pos, &map))
}

/// Draw a hexagon
fn draw_hex(center: na::Vector2<f64>, info: &map::MapInfo) -> Path {
    let basis = get_basis(&info.orientation);

    let points = [
        na::Vector3::new(-1.0,  0.0,  0.0),
        na::Vector3::new( 0.0,  0.0,  1.0),
        na::Vector3::new( 0.0,  1.0,  0.0),
        na::Vector3::new( 1.0,  0.0,  0.0),
        na::Vector3::new( 0.0,  0.0, -1.0),
        na::Vector3::new( 0.0, -1.0,  0.0),
    ];
    let data = Data::new()
        .move_to(point_to_tuple(info.scale * (basis * points[0] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[1] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[2] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[3] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[4] + center)))
        .line_to(point_to_tuple(info.scale * (basis * points[5] + center)))
        .close();

    Path::new()
        .set("d", data)
}

/// Draws a border around a hex
fn draw_hex_edge(center: na::Vector2<f64>, info: &map::MapInfo) -> Path {
    draw_hex(center, info)
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH * info.scale)
}

/// Draws the background (the color) of a hex
fn draw_hex_background(center: na::Vector2<f64>,
                       info: &map::MapInfo,
                       color: tile::colors::Color) -> Path {
    draw_hex(center, info)
        .set("fill", color.value())
        .set("stroke", "none")
}

/// Helper for drawing the paths, does the actual point calculation
fn draw_path_helper(path: &tile::Path,
             center: &na::Vector2<f64>,
             info: &map::MapInfo) -> Path {
    let basis = get_basis(&info.orientation);

    // Calculate end points and control points
    let (start_x, start_y) = point_to_tuple(
        info.scale * (basis * path.start() + center));
    let (end_x, end_y) = point_to_tuple(
        info.scale * (basis * path.end() + center));
    let control1 = C * basis * path.start() + center;
    let control2 = C * basis * path.end() + center;
    let (x1, y1) = point_to_tuple(info.scale * control1);
    let (x2, y2) = point_to_tuple(info.scale * control2);

    // Do the drawing
    let data = Data::new()
        .move_to((start_x, start_y))
        .cubic_curve_to((x1, y1, x2, y2, end_x, end_y));
    Path::new()
        .set("d", data.clone())
        .set("fill", "none")
}

/// Draws the white contrast lines around a path
fn draw_path_contrast(path: &tile::Path,
                      center: &na::Vector2<f64>,
                      info: &map::MapInfo) -> Path {
    draw_path_helper(path, center, info)
        .set("stroke", "white")
        .set("stroke-width", (PATH_WIDTH + 2.0 * LINE_WIDTH) * info.scale)
}

/// Draws the black inside line of a path
fn draw_path(path: &tile::Path,
             center: &na::Vector2<f64>,
             info: &map::MapInfo) -> Group {
    let mut g = Group::new();
    // Draw an outline if the line is a bridge
    if path.is_bridge() {
        g = g.add(draw_path_contrast(path, center, info));
    }
    g.add(draw_path_helper(path, center, info)
          .set("stroke", "black")
          .set("stroke-width", PATH_WIDTH * info.scale))
}

/// Draw a city
fn draw_city(city: tile::City,
             center: na::Vector2<f64>,
             info: &map::MapInfo) -> Group {
    let basis = get_basis(&info.orientation);

    let text_circle_pos = info.scale * (basis * city.revenue_position()
                                        + center);
    let text_pos = text_circle_pos + info.scale *
        na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
    let g = Group::new()
        .add(draw_circle(&text_circle_pos, REVENUE_CIRCLE_RADIUS * info.scale,
                         "white", "black", LINE_WIDTH * info.scale))
        .add(Text::new()
             .add(node::Text::new(city.text_id.to_string()))
             .set("x", text_pos.x)
             .set("y", text_pos.y)
             .set("style", "text-anchor:middle;"));

    let pos = info.scale * (basis * city.position() + center);
    let center = basis * city.position() + center;
    match city.circles {
        1 => g.add(draw_city_circle(&pos, info)),
        2 => {
            let pos1 = pos + na::Vector2::new(-info.scale * TOKEN_SIZE, 0.0);
            let pos2 = pos + na::Vector2::new( info.scale * TOKEN_SIZE, 0.0);
            g.add(Rectangle::new()
                  .set("x", (center.x - TOKEN_SIZE) * info.scale)
                  .set("y", (center.y - TOKEN_SIZE) * info.scale)
                  .set("width", TOKEN_SIZE * info.scale * 2.0)
                  .set("height", TOKEN_SIZE * info.scale * 2.0)
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = info.scale * TOKEN_SIZE;
            let pos1 = pos + na::Vector2::new(0.0, -2.0 * size / sq3);
            let pos2 = pos + na::Vector2::new(-size, size / sq3);
            let pos3 = pos + na::Vector2::new( size, size / sq3);
            let data = Data::new()
                .move_to((-size + pos.x, size / sq3 + size + pos.y))
                .line_to((size + pos.x, size / sq3 + size + pos.y))
                .line_to((size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .line_to((0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .close();
            g.add(Path::new()
                    .set("d", data)
                    .set("fill", "white")
                    .set("stroke", "black")
                    .set("stroke-width", LINE_WIDTH * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
                .add(draw_city_circle(&pos3, info))
        }
        4 => {
            let size = info.scale * TOKEN_SIZE;
            let pos1 = pos + na::Vector2::new(-size, -size);
            let pos2 = pos + na::Vector2::new(-size,  size);
            let pos3 = pos + na::Vector2::new( size, -size);
            let pos4 = pos + na::Vector2::new( size,  size);
            g.add(Rectangle::new()
                  .set("x", (center.x - 2.0 * TOKEN_SIZE) * info.scale)
                  .set("y", (center.y - 2.0 * TOKEN_SIZE) * info.scale)
                  .set("width", TOKEN_SIZE * info.scale * 4.0)
                  .set("height", TOKEN_SIZE * info.scale * 4.0)
                  .set("rx", TOKEN_SIZE * info.scale)
                  .set("fill", "white")
                  .set("stroke", "black")
                  .set("stroke-width", LINE_WIDTH * info.scale))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos1, info))
                .add(draw_city_circle(&pos2, info))
                .add(draw_city_circle(&pos3, info))
                .add(draw_city_circle(&pos4, info))
        }
        x => {
            println!("A tile has an unknown number of circles: {}", x);
            g.add(draw_circle(&pos, TOKEN_SIZE * info.scale, "red", "none",
                              0.0))
        }
    }

}

/// Draw a single city circle
fn draw_city_circle(pos: &na::Vector2<f64>, info: &map::MapInfo) -> Circle {
    draw_circle(pos, TOKEN_SIZE * info.scale, "white", "black",
                LINE_WIDTH * info.scale)
}

fn draw_city_contrast(city: tile::City,
                      center: &na::Vector2<f64>,
                      info: &map::MapInfo) -> Group {
    let basis = get_basis(&info.orientation);
    let g = Group::new();
    let pos = info.scale * (basis * city.position() + center);
    match city.circles {
        1 => {
            let size = (TOKEN_SIZE + LINE_WIDTH) * info.scale;
            g.add(draw_circle(&pos, size, "none", "white",
                              LINE_WIDTH * info.scale))
        }
        2 => {
            let pos = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + LINE_WIDTH) * info.scale,
                (TOKEN_SIZE + LINE_WIDTH) * info.scale);
            g.add(Rectangle::new()
                  .set("x", pos.x)
                  .set("y", pos.y)
                  .set("width",
                       (TOKEN_SIZE * 4.0 + LINE_WIDTH * 2.0) * info.scale)
                  .set("height", (TOKEN_SIZE + LINE_WIDTH) * info.scale * 2.0)
                  .set("rx", TOKEN_SIZE * info.scale)
                  .set("stroke", "white")
                  .set("stroke-width", LINE_WIDTH * info.scale)
            )
        }
        3 => {
            let sq3 = 3.0_f64.sqrt();
            let size = info.scale * TOKEN_SIZE;
            let radius = (TOKEN_SIZE + 1.5 * LINE_WIDTH) * info.scale;
            let pos1 = pos + na::Vector2::new(0.0, -2.0 * size / sq3);
            let pos2 = pos + na::Vector2::new(-size, size / sq3);
            let pos3 = pos + na::Vector2::new( size, size / sq3);
            let data = Data::new()
                .move_to((-size + pos.x, size / sq3 + size + pos.y))
                .line_to((size + pos.x, size / sq3 + size + pos.y))
                .line_to((size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .line_to((0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-0.5 * sq3 * size + pos.x,
                          -2.0 * size / sq3 - 0.5 * size + pos.y))
                .line_to((-size * (0.5 * sq3 + 1.0) + pos.x,
                          (size / sq3 - 0.5 * size) + pos.y))
                .close();
            g.add(Path::new()
                    .set("d", data)
                    .set("stroke", "white")
                    .set("stroke-width", 3.0 *LINE_WIDTH * info.scale))
                .add(draw_circle(&pos1, radius, "white", "none", 0.0))
                .add(draw_circle(&pos2, radius, "white", "none", 0.0))
                .add(draw_circle(&pos3, radius, "white", "none", 0.0))
        }
        4 => {
            let pos = pos - na::Vector2::new(
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * info.scale,
                (2.0 * TOKEN_SIZE + 1.5 * LINE_WIDTH) * info.scale);
            let dim = (4.0 * TOKEN_SIZE + 3.0 * LINE_WIDTH) * info.scale;
            g.add(Rectangle::new()
                  .set("x", pos.x)
                  .set("y", pos.y)
                  .set("width", dim)
                  .set("height", dim)
                  .set("rx", (TOKEN_SIZE + LINE_WIDTH) * info.scale)
                  .set("fill", "white"))
        }
        _ => g,
    }
}

/// Helper to draw circles
fn draw_circle(pos: &na::Vector2<f64>, radius: f64, fill: &str,
               stroke_color: &str, stroke_width: f64) -> Circle {
    Circle::new()
        .set("cx", pos.x)
        .set("cy", pos.y)
        .set("r", radius)
        .set("fill", fill)
        .set("stroke", stroke_color)
        .set("stroke-width", stroke_width)
}

/// Draw a stop
fn draw_stop(stop: tile::Stop,
             center: na::Vector2<f64>,
             info: &map::MapInfo) -> Group {
    let basis = get_basis(&info.orientation);

    let pos = info.scale * (basis * stop.position() + center);
    let a = stop.revenue_angle as f64 * PI / 180.0;
    let text_circle_pos = pos + info.scale *
        na::Matrix2::new(a.cos(), a.sin(), -a.sin(), a.cos()) *
        na::Vector2::new(STOP_TEXT_DIST, 0.0);
    let text_pos = text_circle_pos + info.scale *
        na::Vector2::new(0.0, REVENUE_CIRCLE_RADIUS / 2.5);
    Group::new()
        .add(draw_circle(&pos, STOP_SIZE * info.scale, "black",
                         "white", LINE_WIDTH * info.scale))
        .add(draw_circle(&text_circle_pos, REVENUE_CIRCLE_RADIUS * info.scale,
                         "white", "black", LINE_WIDTH * info.scale))
        .add(Text::new()
             .add(node::Text::new(stop.text_id.to_string()))
             .set("x", text_pos.x)
             .set("y", text_pos.y)
             .set("style", "text-anchor:middle;"))
}

/// Draw a small black circle in the middle of a tile to connect paths nicely
fn draw_lawson(center: na::Vector2<f64>, info: &map::MapInfo) -> Circle {
        // Add LINE_WIDTH to compensate for stroke being half in the circle
    draw_circle(&(center * info.scale),
                (PATH_WIDTH + LINE_WIDTH) * 0.5 * info.scale,
                "black", "white", LINE_WIDTH * info.scale)
}

fn get_basis(orientation: &Orientation) -> na::Matrix2x3<f64> {
    match orientation {
        &Orientation::Horizontal => na::Matrix2x3::from_columns(&[
                na::Vector2::new( 1.0,  0.0),
                na::Vector2::new( 0.5, -0.5 * 3.0_f64.sqrt()),
                na::Vector2::new(-0.5, -0.5 * 3.0_f64.sqrt())
            ]),
        &Orientation::Vertical => na::Matrix2x3::from_columns(&[
                na::Vector2::new(0.5 * 3.0_f64.sqrt(), -0.5),
                na::Vector2::new(0.0, -1.0),
                na::Vector2::new(-0.5 * 3.0_f64.sqrt(), -0.5),
            ]),
    }
}

fn point_to_tuple(p: na::Vector2<f64>) -> (f64, f64) {
    (p.x, p.y)
}
