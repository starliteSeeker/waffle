use std::mem::{self, MaybeUninit};

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
