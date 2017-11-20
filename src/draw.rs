extern crate svg;
extern crate nalgebra as na;

use self::svg::node::element::{Group, Path};
use self::svg::node::element::path::Data;
use super::Orientation;
use super::tile;
use super::map;

/// Draws the tile manifest
pub fn draw_tile_manifest(tiles: &Vec<tile::Tile>) -> Group {
    let tile = draw_tile(&tiles[0], &map::MapInfo::default());

    Group::new()
        .add(tile)
}

/// Draws a single tile
pub fn draw_tile(tile: &tile::Tile, map: &map::MapInfo) -> Group {
    let bg = draw_hex_edge(na::Vector2::new(1.0, 1.0),
                           &map.orientation,
                           None)
        .set("fill", tile.color());

    Group::new()
        .add(bg)
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
        na::Point3::new(-1.0,  0.0,  0.0),
        na::Point3::new( 0.0,  1.0,  0.0),
        na::Point3::new( 0.0,  0.0, -1.0),
        na::Point3::new( 1.0,  0.0,  0.0),
        na::Point3::new( 0.0, -1.0,  0.0),
        na::Point3::new( 0.0,  0.0,  1.0),
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

fn draw_path(center: na::Vector2<f64>,
                 from: na::Point3<f64>,
                 to: na::Point3<f64>,
                 orientation: Orientation,
                 hex_size: Option<f64>) -> Path {
    let id_point3: na::Point3<f64> = na::Point3::new(0.0, 0.0, 0.0);
    let hex_size = match hex_size {
        Some(s) => s,
        None => 20.0,
    };
    let basis = match orientation {
        Orientation::Horizontal => hor_basis(),
        Orientation::Vertical => ver_basis(),
    };
    let (x, y) = point_to_tuple(hex_size * (basis * to + center));
    let (x1, y1) = point_to_tuple(hex_size * (basis * id_point3 + center));
    let data = Data::new()
        .move_to(point_to_tuple(hex_size * (basis * from + center)))
        .quadratic_curve_to((x1, y1, x, y));
    Path::new()
        .set("stroke", "black")
        .set("stroke-width", 2)
        .set("d", data)
}

fn hor_basis() -> na::Matrix2x3<f64> {
    na::Matrix2x3::from_columns(&[
        na::Vector2::new(1.0,   0.0),
        na::Vector2::new(-0.5, -0.5 * 3.0_f64.sqrt()),
        na::Vector2::new(-0.5,  0.5 * 3.0_f64.sqrt())
    ])
}

fn ver_basis() -> na::Matrix2x3<f64> {
    na::Matrix2x3::from_columns(&[
        na::Vector2::new(0.5 * 3.0_f64.sqrt(), -0.5),
        na::Vector2::new(-0.5 * 3.0_f64.sqrt(), -0.5),
        na::Vector2::new(0.0, 1.0)
    ])
}

fn point_to_tuple(p: na::Point2<f64>) -> (f64, f64) {
    (p.coords.x, p.coords.y)
}
