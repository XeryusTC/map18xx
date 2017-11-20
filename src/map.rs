use super::Orientation;
use super::tile::Tile;

pub struct MapInfo {
    name: String,
    pub orientation: Orientation,
    columns: u32,
    rows: u32,
    tiles: Vec<Tile>,
}

impl MapInfo {
    pub fn default () -> MapInfo {
        MapInfo {
            name: String::from("Debug"),
            orientation: Orientation::Horizontal,
            columns: 0,
            rows: 0,
            tiles: vec![],
        }
    }
}
