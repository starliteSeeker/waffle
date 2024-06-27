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
                    data[i].write(Color::new((i as u8 / 16) * 2, (i as u8 % 16) * 2, 0));
                }
                /*
                for i in 0..4 {
                    for j in 0..16 {
                        let ii = i as u32;
                        let jj = j as u32;
                        data[i * 16 + j].write(Color::new(0, 31 * jj / 15, ii * 8 * jj / 15));
                    }
                }
                for i in 4..8 {
                    for j in 0..16 {
                        let ii = i as u32 - 4;
                        let jj = j as u32;
                        data[i * 16 + j].write(Color::new(
                            0,
                            (32 - ii * 8) * jj / 15,
                            31 * jj / 15,
                        ));
                    }
                }
                for i in 8..12 {
                    for j in 0..16 {
                        let ii = i as u32 - 8;
                        let jj = j as u32;
                        data[i * 16 + j].write(Color::new(
                            31 * jj / 15,
                            (32 - ii * 8) * jj / 15,
                            0,
                        ));
                    }
                }
                for i in 12..16 {
                    for j in 0..16 {
                        let ii = i as u32 - 12;
                        let jj = j as u32;
                        data[i * 16 + j].write(Color::new(31 * jj / 15, 0, ii * 8 * jj / 15));
                    }
                }
                */
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
                    let r = lo & 0b00011111;
                    let g = ((lo & 0b11100000) >> 5) | ((hi & 0b00000011) << 3);
                    let b = (hi & 0b01111100) >> 2;
                    data[i].write(Color::new(r, g, b));
                }

                unsafe { mem::transmute::<_, [Color; 256]>(data) }
            },
        })
    }

    pub fn get_curr(&self) -> Color {
        self.pal[self.sel_idx as usize]
    }

    // return true if new value is different
    pub fn set_curr(&mut self, c: Color) -> bool {
        let prev_c = self.pal[self.sel_idx as usize];
        if prev_c == c {
            return false;
        }
        self.pal[self.sel_idx as usize] = c;
        return true;
    }

    pub fn get_relative(&self, idx: u8) -> Color {
        // TODO: assume 2bpp again
        self.pal[(self.sel_idx - (self.sel_idx % 4) + idx) as usize]
    }
}
