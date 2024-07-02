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

#[derive(Sequence, Default)]
pub enum Zoom {
    Half,
    #[default]
    One,
    Two,
}

impl fmt::Display for Zoom {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Zoom::Half => write!(f, "0.5x"),
            Zoom::One => write!(f, "1x"),
            Zoom::Two => write!(f, "2x"),
        }
    }
}

impl Zoom {
    pub fn to_val(&self) -> f64 {
        match self {
            Zoom::Half => 0.5,
            Zoom::One => 1.0,
            Zoom::Two => 2.0,
        }
    }
}

#[derive(Sequence, Default)]
pub enum BGMode {
    #[default]
    M0BG1,
    M0BG2,
    M0BG3,
    M0BG4,
}

impl fmt::Display for BGMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BGMode::M0BG1 => write!(f, "Mode 0 BG1"),
            BGMode::M0BG2 => write!(f, "Mode 0 BG2"),
            BGMode::M0BG3 => write!(f, "Mode 0 BG3"),
            BGMode::M0BG4 => write!(f, "Mode 0 BG4"),
        }
    }
}

impl BGMode {
    pub fn palette_offset(&self) -> u8 {
        match self {
            BGMode::M0BG1 => 0,
            BGMode::M0BG2 => 32,
            BGMode::M0BG3 => 64,
            BGMode::M0BG4 => 96,
        }
    }

    pub fn bpp(&self) -> Bpp {
        match self {
            BGMode::M0BG1 | BGMode::M0BG2 | BGMode::M0BG3 | BGMode::M0BG4 => Bpp::Two,
        }
    }
}