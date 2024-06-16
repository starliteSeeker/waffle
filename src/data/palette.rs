use std::mem::{self, MaybeUninit};

use itertools::Itertools;

use super::color::Color;

pub struct Palette {
    pub pal: [Color; 256],
}

impl Default for Palette {
    fn default() -> Self {
        // unsafe initializing array
        // https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        Palette {
            pal: {
                let mut data: [MaybeUninit<Color>; 256] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                // some random coloful palette
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
            pal: {
                let mut data: [MaybeUninit<Color>; 256] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                for (i, (lo, hi)) in content.into_iter().tuples().enumerate() {
                    let r = lo & 0b00011111;
                    let g = ((lo & 0b11100000) >> 5) | ((hi & 0b00000011) << 3);
                    let b = (hi & 0b01111100) >> 2;
                    data[i].write(Color::new(r as u32, g as u32, b as u32));
                }

                unsafe { mem::transmute::<_, [Color; 256]>(data) }
            },
        })
    }
}
