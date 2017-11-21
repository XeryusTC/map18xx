extern crate svg;
extern crate nalgebra as na;

use std::collections::HashMap;
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
const PATH_WIDTH: f64 = 0.15;
const LINE_WIDTH: f64 = 0.025;
const TOKEN_SIZE: f64 = 0.3;
const STOP_SIZE:  f64 = 0.15;

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

    g = g.add(draw_hex_background(*pos, &map, tile.color()));

    // Draw white contrast lines first
    for path in tile.paths() {
        g = g.add(draw_path_contrast(path, *pos, &map));
    }
    for city in tile.cities() {
        g = g.add(draw_city_contrast(city, pos, &map));
    };

    // Draw elements
    if tile.is_lawson() {
        g = g.add(draw_lawson(*pos, &map));
    }
    for path in tile.paths() {
        g = g.add(draw_path(path, *pos, &map));
    };

    for stop in tile.stops() {
        g = g.add(draw_stop(stop, *pos, &map));
    }

    for city in tile.cities() {
        g = g.add(draw_city(city, *pos, &map));
    };

    g.add(draw_hex_edge(*pos, &map))
}

/// Draw a hexagon
fn draw_hex(center: na::Vector2<f64>, info: &map::MapInfo) -> Path {
    let basis = match &info.orientation {
        &Orientation::Horizontal => hor_basis(),
        &Orientation::Vertical => ver_basis(),
    };

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
fn draw_path_helper(path: tile::Path,
             center: na::Vector2<f64>,
             info: &map::MapInfo) -> Path {
    let basis = match &info.orientation {
        &Orientation::Horizontal => hor_basis(),
        &Orientation::Vertical => ver_basis(),
    };

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
fn draw_path_contrast(path: tile::Path,
                      center: na::Vector2<f64>,
                      info: &map::MapInfo) -> Path {
    draw_path_helper(path, center, info)
        .set("stroke", "white")
        .set("stroke-width", (PATH_WIDTH + 2.0 * LINE_WIDTH) * info.scale)
}

/// Draws the black inside line of a path
fn draw_path(path: tile::Path,
             center: na::Vector2<f64>,
             info: &map::MapInfo) -> Path {
    draw_path_helper(path, center, info)
        .set("stroke", "black")
        .set("stroke-width", PATH_WIDTH * info.scale)
}

/// Draw a city
fn draw_city(city: tile::City,
             center: na::Vector2<f64>,
             info: &map::MapInfo) -> Group {
    let basis = match &info.orientation {
        &Orientation::Horizontal => hor_basis(),
        &Orientation::Vertical => ver_basis(),
    };
    let g = Group::new();

    let pos = info.scale * (basis * city.position() + center);
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
        x => {
            println!("A tile has an unknown number of circles: {}", x);
            g.add(Circle::new()
                  .set("cx", pos.x)
                  .set("cy", pos.y)
                  .set("r", TOKEN_SIZE * info.scale)
                  .set("fill", "red"))
        }
    }

}

/// Draw a single city circle
fn draw_city_circle(pos: &na::Vector2<f64>, info: &map::MapInfo) -> Circle {
    Circle::new()
        .set("cx", pos.x)
        .set("cy", pos.y)
        .set("r", TOKEN_SIZE * info.scale)
        .set("fill", "white")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH * info.scale)
}

fn draw_city_contrast(city: tile::City,
                      center: &na::Vector2<f64>,
                      info: &map::MapInfo) -> Group {
    let basis = match &info.orientation {
        &Orientation::Horizontal => hor_basis(),
        &Orientation::Vertical => ver_basis(),
    };
    let g = Group::new();
    let pos = info.scale * (basis * city.position() + center);
    match city.circles {
        1 => {
            let size = (TOKEN_SIZE + LINE_WIDTH) * info.scale;
            g.add(Circle::new()
                  .set("cx", pos.x)
                  .set("cy", pos.y)
                  .set("r", size)
                  .set("stroke", "white")
                  .set("stroke-width", LINE_WIDTH * info.scale))
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
        _ => g,
    }
}

/// Draw a stop
fn draw_stop(stop: tile::Stop,
             center: na::Vector2<f64>,
             info: &map::MapInfo) -> Circle {
    let basis = match &info.orientation {
        &Orientation::Horizontal => hor_basis(),
        &Orientation::Vertical => ver_basis(),
    };

    let pos = info.scale * (basis * stop.position() + center);
    Circle::new()
        .set("cx", pos.x)
        .set("cy", pos.y)
        .set("r", STOP_SIZE * info.scale)
        .set("fill", "black")
        .set("stroke", "white")
        .set("stroke-width", LINE_WIDTH * info.scale)
}

/// Draw a small black circle in the middle of a tile to connect paths nicely
fn draw_lawson(center: na::Vector2<f64>, info: &map::MapInfo) -> Circle {
    Circle::new()
        .set("cx", center.x * info.scale)
        .set("cy", center.y * info.scale)
        // Add LINE_WIDTH to compensate for stroke being half in the circle
        .set("r", (PATH_WIDTH + LINE_WIDTH) / 2.0 * info.scale)
        .set("fill", "black")
        .set("stroke", "white")
        .set("stroke-width", LINE_WIDTH * info.scale)
}

fn hor_basis() -> na::Matrix2x3<f64> {
    na::Matrix2x3::from_columns(&[
        na::Vector2::new( 1.0,  0.0),
        na::Vector2::new( 0.5, -0.5 * 3.0_f64.sqrt()),
        na::Vector2::new(-0.5, -0.5 * 3.0_f64.sqrt())
    ])
}

fn ver_basis() -> na::Matrix2x3<f64> {
    na::Matrix2x3::from_columns(&[
        na::Vector2::new(0.5 * 3.0_f64.sqrt(), -0.5),
        na::Vector2::new(0.0, -1.0),
        na::Vector2::new(-0.5 * 3.0_f64.sqrt(), -0.5),
    ])
}

fn point_to_tuple(p: na::Vector2<f64>) -> (f64, f64) {
    (p.x, p.y)
}
