use modular_bitfield::prelude::*;

#[bitfield]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct Color {
    pub red: B5,
    pub green: B5,
    pub blue: B5,
    #[skip]
    __: B1,
}

impl Color {
    pub fn to_tuple(&self) -> (u8, u8, u8) {
        (self.red(), self.green(), self.blue())
    }

    pub fn to_cairo(&self) -> (f64, f64, f64) {
        // convert range 0-31 to 0.0-1.0
        (
            self.red() as f64 / 31.0,
            self.green() as f64 / 31.0,
            self.blue() as f64 / 31.0,
        )
    }
}
