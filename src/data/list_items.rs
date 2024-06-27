use std::fmt;

use enum_iterator::Sequence;

#[derive(Sequence)]
pub enum Bpp {
    Two = 2,
    Four = 4,
}

impl fmt::Display for Bpp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bpp::Two => write!(f, "2bpp"),
            Bpp::Four => write!(f, "4bpp"),
        }
    }
}

#[derive(Sequence, Default)]
pub enum TileSize {
    #[default]
    Eight = 8,
    Sixteen = 16,
}

impl fmt::Display for TileSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TileSize::Eight => write!(f, "8x8"),
            TileSize::Sixteen => write!(f, "16x16"),
        }
    }
}
