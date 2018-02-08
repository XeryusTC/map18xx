extern crate nalgebra as na;

use super::svg::node::element;
use super::svg::node::element::path::Data;
use draw::consts::*;
use game;
use game::Orientation;
use tile;

/// Calculate scale for a map
pub fn scale(map: &game::Map) -> f64 {
    map.scale * PPCM
}

/// Draw a hexagon
fn draw_hex(center: na::Vector2<f64>,
                info: &game::Map) -> element::Path {
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
        .move_to(point_to_tuple(scale(&info) * (basis * points[0] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[1] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[2] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[3] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[4] + center)))
        .line_to(point_to_tuple(scale(&info) * (basis * points[5] + center)))
        .close();

    element::Path::new()
        .set("d", data)
}

/// Draws a border around a hex
pub fn draw_hex_edge(center: na::Vector2<f64>,
                     info: &game::Map) -> element::Path {
    draw_hex(center, info)
        .set("fill", "none")
        .set("stroke", "black")
        .set("stroke-width", LINE_WIDTH * scale(&info))
}

/// Draws the background (the color) of a hex
pub fn draw_hex_background(center: na::Vector2<f64>,
                       info: &game::Map,
                       color: tile::colors::Color) -> element::Path {
    draw_hex(center, info)
        .set("fill", color.value())
        .set("stroke", "none")
}

/// Helper for drawing the paths, does the actual point calculation
pub fn draw_path_helper(path: &tile::Path,
                        center: &na::Vector2<f64>,
                        info: &game::Map,
                        rotation: &f64) -> element::Path {
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);

    // Calculate end points and control points
    let (start_x, start_y) = point_to_tuple(
        scale(&info) * (rot * basis * path.start() + center));
    let (end_x, end_y) = point_to_tuple(
        scale(&info) * (rot * basis * path.end() + center));
    let control1 = match &path.start_control {
        &None => path.radius() * C * rot * basis * path.start() + center,
        &Some(ref point) => rot * basis * point.as_vector() + center,
    };
    let control2 = match &path.end_control {
        &None => path.radius() * C * rot * basis * path.end() + center,
        &Some(ref point) => rot * basis * point.as_vector() + center,
    };
    let (x1, y1) = point_to_tuple(scale(&info) * control1);
    let (x2, y2) = point_to_tuple(scale(&info) * control2);

    // Do the drawing
    let data = Data::new()
        .move_to((start_x, start_y))
        .cubic_curve_to((x1, y1, x2, y2, end_x, end_y));
    element::Path::new()
        .set("d", data.clone())
        .set("fill", "none")
}

/// Draw a single city circle
pub fn draw_city_circle(pos: &na::Vector2<f64>,
                        info: &game::Map) -> element::Circle {
    draw_circle(pos, TOKEN_SIZE * scale(&info), "white", "black",
                LINE_WIDTH * scale(&info))
}

/// Calculate the position of a single circle in a city
pub fn city_circle_pos(city: &tile::City,
                   circle: u32,
                   center: &na::Vector2<f64>,
                   info: &game::Map,
                   rotation: &f64) -> na::Vector2<f64> {
    let basis = get_basis(&info.orientation);
    let rot = rotate(rotation);
    let pos = rot * basis * city.position() + center;
    let pos = match city.circles {
        1 => pos,
        2 => match circle {
            0 => pos - rot * na::Vector2::new(TOKEN_SIZE, 0.0),
            1 => pos + rot * na::Vector2::new(TOKEN_SIZE, 0.0),
            n => panic!("Illegal circle id {} for city of size 2", n),
        },
        3 => match circle {
            0 => pos + na::Vector2::new(0.0, -2.0 * TOKEN_SIZE/3.0_f64.sqrt()),
            1 => pos + na::Vector2::new(-TOKEN_SIZE,
                                        TOKEN_SIZE / 3.0_f64.sqrt()),
            2 => pos + na::Vector2::new(TOKEN_SIZE, TOKEN_SIZE/3.0_f64.sqrt()),
            n => panic!("Illegal circle id {} for city of size 3", n),
        },
        4 => match circle {
            0 => pos + na::Vector2::new(-TOKEN_SIZE, -TOKEN_SIZE),
            1 => pos + na::Vector2::new(-TOKEN_SIZE,  TOKEN_SIZE),
            2 => pos + na::Vector2::new( TOKEN_SIZE, -TOKEN_SIZE),
            3 => pos + na::Vector2::new( TOKEN_SIZE,  TOKEN_SIZE),
            n => panic!("Illegal circle id {} for city of size 3", n),
        }
        n => panic!("Cities of {} not supported!", n),
    };
    pos * scale(&info)
}

/// Helper to draw circles
pub fn draw_circle(pos: &na::Vector2<f64>, radius: f64, fill: &str,
                   stroke_color: &str, stroke_width: f64) -> element::Circle {
    element::Circle::new()
        .set("cx", pos.x)
        .set("cy", pos.y)
        .set("r", radius)
        .set("fill", fill)
        .set("stroke", stroke_color)
        .set("stroke-width", stroke_width)
}

pub fn get_basis(orientation: &Orientation) -> na::Matrix2x3<f64> {
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

pub fn rotate(theta: &f64) -> na::Matrix2<f64> {
    na::Matrix2::new(theta.cos(), -theta.sin(), theta.sin(), theta.cos())
}

pub fn point_to_tuple(p: na::Vector2<f64>) -> (f64, f64) {
    (p.x, p.y)
}
