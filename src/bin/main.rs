extern crate svg;
extern crate nalgebra as na;
extern crate map18xx;

use svg::Document;
use map18xx::draw;
use map18xx::tile::{colors, Tile};

fn main() {
    let tile8 = Tile::new(String::from("8"), colors::YELLOW);
    let document = Document::new()
        .set("viewBox", (0, 0, 100, 100))
        .add(draw::draw_tile_manifest(&vec![tile8]));
    svg::save("image.svg", &document).unwrap();
}
