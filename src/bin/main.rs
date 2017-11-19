extern crate svg;
extern crate nalgebra as na;
extern crate map18xx;

use svg::Document;
use map18xx::draw;
use map18xx::Orientation;

fn main() {
    let hex = draw::draw_hex_edge(na::Vector2::new(1.0, 1.0),
                                   Orientation::Horizontal,
                                   None)
        .set("fill", "yellow");
    let path = draw::draw_path(na::Vector2::new(1.0, 1.0),
                               na::Point3::new(0.0, -0.5, 0.5),
                               na::Point3::new(0.5, 0.0, -0.5),
                               Orientation::Horizontal,
                               None)
        .set("fill", "none");
    let document = Document::new()
        .set("width", 100)
        .set("height", 100)
        .set("viewBox", (0, 0, 70, 70))
        .add(hex)
        .add(path);
    svg::save("image.svg", &document).unwrap();
}
