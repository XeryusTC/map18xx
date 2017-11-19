extern crate svg;
extern crate nalgebra as na;
extern crate map18xx;

use svg::Document;
use map18xx::draw;
use map18xx::Orientation;

fn main() {
    let path = draw::draw_hex_edge(na::Vector2::new(1.0, 1.0),
                                   Orientation::Horizontal,
                                   None)
        .set("fill", "none");
    let document = Document::new()
        .set("viewBox", (0, 0, 70, 70))
        .add(path);
    svg::save("image.svg", &document).unwrap();
}
