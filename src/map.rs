use super::Orientation;

pub struct MapInfo {
    pub orientation: Orientation,
    pub scale: f64,
}

impl MapInfo {
    pub fn default () -> MapInfo {
        MapInfo {
            orientation: Orientation::Horizontal,
            scale: 3.81, // Hexes are usually 3.81cm flat-to-flat
        }
    }
}
