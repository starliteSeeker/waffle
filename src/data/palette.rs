use std::fs::File;
use std::io::Write;
use std::mem::{self, MaybeUninit};

use itertools::Itertools;

use super::color::Color;

pub struct Palette(pub [Color; 256]);

impl Default for Palette {
    fn default() -> Self {
        Self({
            // unsafe initializing array
            // https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
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
        })
    }
}

impl Palette {
    pub fn from_file_bgr555(path: &std::path::PathBuf) -> std::io::Result<Self> {
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
        Ok(Self({
            let mut data: [MaybeUninit<Color>; 256] =
                unsafe { MaybeUninit::uninit().assume_init() };

            for (i, (lo, hi)) in content.into_iter().tuples().enumerate() {
                data[i].write(Color::from_bytes([lo, hi]));
            }

            unsafe { mem::transmute::<_, [Color; 256]>(data) }
        }))
    }

    pub fn write_file_bgr555(&self, mut file: &File) -> std::io::Result<()> {
        for c in &self.0 {
            file.write_all(&c.into_bytes())?;
        }
        Ok(())
    }

    pub fn from_file_rgb24(path: &std::path::PathBuf) -> std::io::Result<Self> {
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
        Ok(Self({
            let mut data: [MaybeUninit<Color>; 256] =
                unsafe { MaybeUninit::uninit().assume_init() };

            for (i, (r, g, b)) in content.into_iter().tuples().enumerate() {
                let r = r >> 3;
                let g = g >> 3;
                let b = b >> 3;
                data[i].write(Color::new().with_red(r).with_green(g).with_blue(b));
            }

            unsafe { mem::transmute::<_, [Color; 256]>(data) }
        }))
    }

    pub fn write_file_rgb24(&self, mut file: &File) -> std::io::Result<()> {
        for c in &self.0 {
            let r = c.red() << 3 | c.red() >> 2;
            let g = c.green() << 3 | c.green() >> 2;
            let b = c.blue() << 3 | c.blue() >> 2;
            let _ = file.write_all(&[r, g, b])?;
        }
        Ok(())
    }
}
