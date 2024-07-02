use std::mem::{self, MaybeUninit};

use itertools::Itertools;

use super::color::Color;

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
                for i in 0..256 {
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
    pub fn from_path(path: &std::path::PathBuf) -> std::io::Result<Palette> {
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

    pub fn get_curr(&self) -> Color {
        self.pal[self.sel_idx as usize]
    }

    // return true if new value is different
    pub fn set_curr(&mut self, r: u8, g: u8, b: u8) -> bool {
        let prev_c = self.pal[self.sel_idx as usize];
        if prev_c.red() == r && prev_c.green() == g && prev_c.blue() == b {
            return false;
        }
        self.pal[self.sel_idx as usize] = Color::new()
            .with_red(r.min(31))
            .with_green(g.min(31))
            .with_blue(b.min(31));
        return true;
    }

    pub fn get_relative(&self, idx: u8) -> Color {
        // TODO: assume 2bpp again
        self.pal[(self.sel_idx - (self.sel_idx % 4) + idx) as usize]
    }
}
