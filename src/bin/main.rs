extern crate map18xx;
extern crate svg;

use map18xx::{draw, tile};
use svg::Document;

fn main() {
    let definitions = tile::definitions();
    let mut document = Document::new()
        .set("viewBox", (0, 0, 65, 40 * definitions.len()))
        .add(draw::draw_tile_definitions(&definitions));

    svg::save("definitions.svg", &document).unwrap();
}
