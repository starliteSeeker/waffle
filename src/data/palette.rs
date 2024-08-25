use std::mem::{self, MaybeUninit};

use itertools::Itertools;

use super::color::Color;
use crate::data::list_items::BGMode;

pub struct Palette {
    pub sel_idx: u8,
    pub pal: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        // unsafe initializing array
        // https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        Palette {
            sel_idx: 0,
            pal: {
                let mut data: [MaybeUninit<Color>; 256] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                // some random coloful palette
                for i in 0..8 {
                    for j in 0..16 {
                        let r = if i & 0b001 == 0 {
                            (j as u8 * 2).min(31)
                        } else {
                            0
                        };
                        let g = if i & 0b010 == 0 {
                            (j as u8 * 2).min(31)
                        } else {
                            0
                        };
                        let b = if i & 0b100 == 0 {
                            (j as u8 * 2).min(31)
                        } else {
                            0
                        };
                        data[i * 16 + j]
                            .write(Color::from_bytes([r | (g & 0b111) << 5, g >> 3 | b << 2]));
                    }
                }
                for i in 128..256 {
                    let r = i as u8 & 0b00001111;
                    let g = (i as u8 & 0b01110000) >> 4;
                    let b = if i & 0x80 != 0 { 0b11111 } else { 0b0 };
                    data[i].write(Color::from_bytes([
                        r << 1 | (g & 0b1) << 7,
                        g >> 1 | b << 2,
                    ]));
                }
                unsafe { mem::transmute::<_, [Color; 256]>(data) }
            },
        }
    }
}

impl Palette {
    pub fn from_path_bgr555(path: &std::path::PathBuf) -> std::io::Result<Palette> {
        let mut content = std::fs::read(&path)?;
        let len = content.len();
        if len < 512 {
            eprintln!("file size less than 512B, pad with 0");
        } else if len > 512 {
            eprintln!("file size greater than 512B, trim extra bytes");
        }
        content.resize(512, 0);

        // unsafe initializing array
        // https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        Ok(Palette {
            sel_idx: 0,
            pal: {
                let mut data: [MaybeUninit<Color>; 256] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                for (i, (lo, hi)) in content.into_iter().tuples().enumerate() {
                    data[i].write(Color::from_bytes([lo, hi]));
                }

                unsafe { mem::transmute::<_, [Color; 256]>(data) }
            },
        })
    }

    pub fn from_path_rgb24(path: &std::path::PathBuf) -> std::io::Result<Palette> {
        let content = std::fs::read(&path)?;
        let len = content.len();
        if len != 3 * 256 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "file must be 768 bytes",
            ));
        }

        // unsafe initializing array
        // https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        Ok(Palette {
            sel_idx: 0,
            pal: {
                let mut data: [MaybeUninit<Color>; 256] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                for (i, (r, g, b)) in content.into_iter().tuples().enumerate() {
                    let r = r >> 3;
                    let g = g >> 3;
                    let b = b >> 3;
                    data[i].write(Color::new().with_red(r).with_green(g).with_blue(b));
                }

                unsafe { mem::transmute::<_, [Color; 256]>(data) }
            },
        })
    }

    pub fn curr_color(&self) -> &Color {
        &self.pal[self.sel_idx as usize]
    }

    // return true if new value is different
    pub fn set_curr(&mut self, r: u8, g: u8, b: u8) -> bool {
        let prev_c = self.curr_color();
        if prev_c.red() == r && prev_c.green() == g && prev_c.blue() == b {
            return false;
        }
        self.pal[self.sel_idx as usize] = Color::new()
            .with_red(r.min(31))
            .with_green(g.min(31))
            .with_blue(b.min(31));
        return true;
    }

    pub fn get_relative(&self, color_idx: u8, bg_mode: &BGMode) -> Color {
        // let i = bg_mode.palette_offset() + self.pal_sel * bg_mode.bpp().to_val() + color_idx;
        let i = self.sel_idx - (self.sel_idx % bg_mode.bpp().to_val()) + color_idx;
        self.pal[i as usize]
    }

    pub fn set_idx(&mut self, idx: u8) -> bool {
        if idx == self.sel_idx {
            return false;
        }
        self.sel_idx = idx;
        return true;
    }
}
