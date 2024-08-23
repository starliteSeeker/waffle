use strum::EnumString;

#[derive(EnumString, Debug, Copy, Clone, Default, Eq, PartialEq)]
pub enum PaletteFile {
    #[default]
    BGR555,
    RGB24,
}
