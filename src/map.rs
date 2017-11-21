use super::Orientation;

pub struct MapInfo {
    pub orientation: Orientation,
    pub scale: f64,
}

impl MapInfo {
    pub fn default () -> MapInfo {
        MapInfo {
            orientation: Orientation::Horizontal,
            scale: 20.0,
        }
    }
}
