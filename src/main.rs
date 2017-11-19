extern crate svg;
extern crate nalgebra as na;

use svg::Document;
use svg::node::element::Path;
use svg::node::element::path::Data;

enum Orientation {
    Horizontal,
    Vertical,
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

fn main() {
    let path = draw_hex(na::Vector2::new(1.0, 1.0),
                        Orientation::Horizontal,
                        None)
        .set("fill", "none");
    let document = Document::new()
        .set("viewBox", (0, 0, 70, 70))
        .add(path);
    svg::save("image.svg", &document).unwrap();
}

fn draw_hex(center: na::Vector2<f64>,
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

fn point_to_tuple(p: na::Point2<f64>) -> (f64, f64) {
    (p.coords.x, p.coords.y)
}
