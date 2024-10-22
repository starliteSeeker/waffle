use std::fmt;

use strum::{EnumIter, EnumString};

use gtk::glib;

#[derive(EnumString, EnumIter, Default, Debug, PartialEq, Eq, Copy, Clone, glib::Enum)]
#[enum_type(name = "Bpp")]
pub enum Bpp {
    #[default]
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

#[derive(EnumIter, Default, Debug, Copy, Clone, glib::Enum)]
#[enum_type(name = "TileSize")]
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

#[derive(EnumIter, Default, Debug, Copy, Clone, glib::Enum)]
#[enum_type(name = "Zoom")]
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

#[derive(EnumIter, Default, Debug, Copy, Clone, glib::Enum)]
#[enum_type(name = "BGModeTwo")]
pub enum BGModeTwo {
    #[default]
    M0BG1,
    M0BG2,
    M0BG3,
    M0BG4,
}

impl fmt::Display for BGModeTwo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BGModeTwo::M0BG1 => write!(f, "Mode 0 BG1"),
            BGModeTwo::M0BG2 => write!(f, "Mode 0 BG2"),
            BGModeTwo::M0BG3 => write!(f, "Mode 0 BG3"),
            BGModeTwo::M0BG4 => write!(f, "Mode 0 BG4"),
        }
    }
}

impl BGModeTwo {
    pub fn palette_offset(&self) -> u8 {
        match self {
            BGModeTwo::M0BG1 => 0,
            BGModeTwo::M0BG2 => 32,
            BGModeTwo::M0BG3 => 64,
            BGModeTwo::M0BG4 => 96,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum DrawMode {
    #[default]
    None,
    Pen,
    RectFill {
        start: (usize, usize),
        end: (usize, usize),
    },
}

impl DrawMode {
    pub fn idx_in_range(&self, ix: usize, iy: usize) -> bool {
        match self {
            DrawMode::RectFill { start, end } => {
                let ((x_min, x_max), (y_min, y_max)) = (
                    (start.0.min(end.0), start.0.max(end.0)),
                    (start.1.min(end.1), start.1.max(end.1)),
                );
                ix >= x_min && ix <= x_max && iy >= y_min && iy <= y_max
            }
            DrawMode::Pen => false,
            DrawMode::None => false,
        }
    }
}
