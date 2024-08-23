use std::fmt;

use strum::EnumIter;

#[derive(EnumIter)]
pub enum Bpp {
    Two,
    Four,
}

impl fmt::Display for Bpp {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Bpp::Two => write!(f, "2bpp"),
            Bpp::Four => write!(f, "4bpp"),
        }
    }
}

impl Bpp {
    pub fn bits(&self) -> u8 {
        match self {
            Bpp::Two => 2,
            Bpp::Four => 4,
        }
    }

    pub fn to_val(&self) -> u8 {
        match self {
            Bpp::Two => 4,
            Bpp::Four => 16,
        }
    }
}

#[derive(EnumIter, Default)]
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

#[derive(EnumIter, Default)]
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

#[derive(EnumIter, Default)]
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
    // which part of the palette is used is decided by BGMode
    // 4bpp backgrounds use colors 0-127
    // 2bpp backgrounds use a range of 32 colors starting from palette_offset()
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

    pub fn idx_to_pal_sel(&self, mut idx: u8) -> Option<u8> {
        if idx < self.palette_offset() {
            return None;
        }
        idx -= self.palette_offset();
        idx /= self.bpp().to_val();
        if idx >= 8 {
            return None;
        }
        return Some(idx);
    }
}
