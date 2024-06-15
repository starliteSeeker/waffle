#[derive(Debug, Copy, Clone, Default)]
pub struct Color {
    red: u32,
    green: u32,
    blue: u32,
}

impl Color {
    pub fn new(r: u32, g: u32, b: u32) -> Self {
        // clamp rgb values to be between 0 and 31 (0b11111)
        Color {
            red: r.min(31),
            green: g.min(31),
            blue: b.min(31),
        }
    }

    pub fn to_tuple(&self) -> (u32, u32, u32) {
        (self.red, self.green, self.blue)
    }

    pub fn to_cairo(&self) -> (f64, f64, f64) {
        // convert range 0-31 to 0.0-1.0
        (
            self.red as f64 / 31.0,
            self.green as f64 / 31.0,
            self.blue as f64 / 31.0,
        )
    }
}
