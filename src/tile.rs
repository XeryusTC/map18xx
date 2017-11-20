pub mod colors {
    pub struct Color {
        value: &'static str,
    }

    impl Color {
        pub fn value(&self) -> &str {
            self.value
        }
    }

    pub const GROUND:  Color  = Color { value: "#F5F5F5" };
    pub const YELLOW:  Color  = Color { value: "#FFFF00" };
    pub const GREEN:   Color  = Color { value: "#64E164" };
    pub const RUSSET:  Color  = Color { value: "#EE7621" };
    pub const GREY:    Color  = Color { value: "#BEBEBE" };
    pub const BROWN:   Color  = Color { value: "#CD6600" };
    pub const RED:     Color  = Color { value: "#FF6464" };
    pub const BLUE:    Color  = Color { value: "#6464FF" };
    pub const BARRIER: Color  = Color { value: "#1E90FF" };
    pub const WHITE:   Color  = Color { value: "#FFFFFF" };
}

pub struct Tile {
    number: String,
    color: colors::Color,
}

impl Tile {
    pub fn new(number: String, color: colors::Color) -> Tile {
        Tile { number, color }
    }

    pub fn color(&self) -> &str {
        self.color.value()
    }
}
