use gtk::glib::{FixedSizeVariantArray, Variant, VariantTy};
use gtk::prelude::*;
use modular_bitfield::prelude::*;
use std::borrow::Cow;

#[bitfield]
#[derive(PartialEq, Eq, Debug, Copy, Clone, Default)]
pub struct Color {
    pub red: B5,
    pub green: B5,
    pub blue: B5,
    #[skip]
    __: B1,
}

impl StaticVariantType for Color {
    fn static_variant_type() -> Cow<'static, VariantTy> {
        <FixedSizeVariantArray<[u8; 2], u8>>::static_variant_type()
    }
}

impl ToVariant for Color {
    fn to_variant(&self) -> Variant {
        Variant::array_from_fixed_array(&self.into_bytes())
    }
}

impl FromVariant for Color {
    fn from_variant(variant: &Variant) -> Option<Self> {
        let s = Self::from_bytes(
            variant
                .fixed_array()
                .expect("color variant type mismatch")
                .try_into()
                .expect("color variant bytearray mismatch"),
        );
        Some(s)
    }
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
