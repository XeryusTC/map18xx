extern crate svg;
extern crate nalgebra as na;

use self::svg::node::element::Path;
use self::svg::node::element::path::Data;
use super::Orientation;

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
pub fn draw_hex_edge(center: na::Vector2<f64>,
            orientation: Orientation,
            hex_size: Option<f64>) -> Path {
    let hex_size = match hex_size {
        Some(s) => s,
        None => 20.0,
    };
    let basis = match orientation {
        Orientation::Horizontal => hor_basis(),
        Orientation::Vertical => ver_basis(),
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
