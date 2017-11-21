extern crate svg;
extern crate nalgebra as na;

use std::collections::HashMap;
use self::svg::node;
use self::svg::node::element::{Circle, Group, Path, Text};
use self::svg::node::element::path::Data;
use super::Orientation;
use super::tile;
use super::tile::TileSpec;
use super::map;

const C: f64 = 0.551915024494;

/// Draws tile definitions
pub fn draw_tile_definitions(
        definitions: &HashMap<String, tile::TileDefinition>) -> Group {
    let mut g = Group::new();
    let mut i = 0.0;
    let info = map::MapInfo::default();
    for (name, definition) in definitions {
        println!("Rendering {}: {}", i, name);
        let drawing = draw_tile(&definition,
                                &na::Vector2::new(1.1, 1.0 + 2.0 * i),
                                &info)
            .set("fill", "white");
        let text = Text::new()
            .add(node::Text::new(name.as_str()))
            .set("x", 50)
            .set("y", 20.0* (1.0 + 2.0 * i));
        g = g.add(drawing).add(text);
        i += 1.0;
    }
    g
}

/// Draws a single tile
pub fn draw_tile(tile: &tile::TileDefinition,
                 pos: &na::Vector2<f64>,
                 map: &map::MapInfo) -> Group {
    let mut g = Group::new();

    let bg = draw_hex_edge(*pos, &map.orientation, None)
        .set("fill", tile.color().value());
    g = g.add(bg);

    for path in tile.paths() {
        g = g.add(draw_path(path, *pos, &map.orientation, None));
    };

    for city in tile.cities() {
        g = g.add(draw_city(city, *pos, &map.orientation, None));
    };

    g
}

/// Draw the outline of a hexagon
///
/// # Parameters
///
/// center: the middle point of the hex
///
/// orientation: whether the hex should have a flat top or one of the points
///              should be at the top
///
/// hex_size: a factor to scale the hex by
fn draw_hex_edge(center: na::Vector2<f64>,
            orientation: &Orientation,
            hex_size: Option<f64>) -> Path {
    let hex_size = match hex_size {
        Some(s) => s,
        None => 20.0,
    };
    let basis = match orientation {
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
        .move_to(point_to_tuple(hex_size * (basis * points[0] + center)))
        .line_to(point_to_tuple(hex_size * (basis * points[1] + center)))
        .line_to(point_to_tuple(hex_size * (basis * points[2] + center)))
        .line_to(point_to_tuple(hex_size * (basis * points[3] + center)))
        .line_to(point_to_tuple(hex_size * (basis * points[4] + center)))
        .line_to(point_to_tuple(hex_size * (basis * points[5] + center)))
        .close();

    Path::new()
        .set("stroke", "black")
        .set("stroke-width", 0.5)
        .set("d", data)
}

fn draw_path(path: tile::Path,
             center: na::Vector2<f64>,
             orientation: &Orientation,
             hex_size: Option<f64>) -> Path {
    let hex_size = match hex_size {
        Some(s) => s,
        None => 20.0,
    };
    let basis = match orientation {
        &Orientation::Horizontal => hor_basis(),
        &Orientation::Vertical => ver_basis(),
    };
    let (start_x, start_y) = point_to_tuple(
        hex_size * (basis * path.start() + center));
    let (end_x, end_y) = point_to_tuple(
        hex_size * (basis * path.end() + center));
    let control1 = C * basis * path.start() + center;
    let control2 = C * basis * path.end() + center;
    let (x1, y1) = point_to_tuple(hex_size * control1);
    let (x2, y2) = point_to_tuple(hex_size * control2);
    let data = Data::new()
        .move_to((start_x, start_y))
        .cubic_curve_to((x1, y1, x2, y2, end_x, end_y));
    Path::new()
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("d", data)
}

fn draw_city(city: tile::City,
             center: na::Vector2<f64>,
             orientation: &Orientation,
             hex_size: Option<f64>) -> Circle {
    let hex_size = match hex_size {
        Some(s) => s,
        None => 20.0,
    };
    let basis = match orientation {
        &Orientation::Horizontal => hor_basis(),
        &Orientation::Vertical => ver_basis(),
    };

    let pos = hex_size * (basis * city.position() + center);
    Circle::new()
        .set("cx", pos.x)
        .set("cy", pos.y)
        .set("r", hex_size * 0.3)
        .set("fill", "white")
        .set("stroke", "black")
        .set("stroke-width", 0.5)
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
