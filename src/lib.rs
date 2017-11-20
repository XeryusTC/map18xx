#[macro_use]
extern crate serde_derive;

pub mod draw;
pub mod tile;
pub mod map;

/// Orientation that hexes should be in
pub enum Orientation {
    /// Hexes should have a flat top
    Horizontal,
    /// Hexes should have apoint at the top
    Vertical,
}
