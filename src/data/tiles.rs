use crate::data::palette::Palette;
use std::cell::RefCell;
use std::rc::Rc;

use crate::data::list_items::BGMode;

pub struct Tile {
    chr: [u8; 64],
}

impl Tile {
    fn from_2bpp(s: &[u8]) -> Option<Self> {
        // 8 * 8 pixels * 2 (bits/pixel) / 8 (bits/byte)
        if s.len() != 16 {
            return None;
        }

        let mut chr = [0; 64];
        for i in 0..64 {
            // pxl 01234567
            //   0bxxxxxxxx
            let a = i / 8;
            let b = i % 8;
            chr[i] = (s[2 * a] >> (7 - b)) & 0b1; // bit 0
            chr[i] |= ((s[2 * a + 1] >> (7 - b)) & 0b1) << 1; // bit 1
        }
        Some(Tile { chr })
    }

    pub fn draw(
        &self,
        cr: &gtk::cairo::Context,
        palette_data: Rc<RefCell<Palette>>,
        palette_offset: Option<u8>,
        bg_mode: &BGMode,
    ) {
        let pxl_w = crate::TILE_W / 8.0;
        // TODO: assume 2bpp for now
        // collect pixels with same color, then draw the pixels together
        let mut rects = vec![Vec::new(); 4];

        // (0, 0) as top left corner of tile
        for (j, c) in self.chr.into_iter().enumerate() {
            // top left corner of pixel
            let x_off = (j % 8) as f64 * pxl_w;
            let y_off = (j / 8) as f64 * pxl_w;
            rects[c as usize].push((x_off, y_off));
        }

        for (i, v) in rects.into_iter().enumerate() {
            for (x, y) in v {
                cr.rectangle(x, y, pxl_w, pxl_w);
            }
            let (r, g, b) = if let Some(idx) = palette_offset {
                palette_data.borrow().pal[idx as usize + i].to_cairo()
            } else {
                palette_data
                    .borrow()
                    .get_relative(i as u8, bg_mode)
                    .to_cairo()
            };
            cr.set_source_rgb(r, g, b);
            let _ = cr.fill();
        }
    }
}

pub struct Tileset {
    sel_idx: u32,
    pub tiles: Vec<Tile>,
}

impl Default for Tileset {
    fn default() -> Self {
        Tileset {
            sel_idx: 0,
            tiles: vec![
                Tile { chr: [0; 64] },
                Tile { chr: [1; 64] },
                Tile { chr: [2; 64] },
                Tile { chr: [3; 64] },
            ],
        }
    }
}

impl Tileset {
    pub fn from_path(path: &std::path::PathBuf) -> std::io::Result<Self> {
        let mut content = std::fs::read(path)?;
        let len = content.len();
        if len == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "file has length 0",
            ));
        }
        let align = 8 * 8 * 2 / 8;
        if len % align != 0 {
            eprintln!("file does not align with {align} bytes, pad with 0");
            content.resize(len + align - (len % align), 0);
        }

        let mut tiles = Vec::new();
        for i in (0..len).step_by(align) {
            tiles.push(Tile::from_2bpp(&content[i..i + align]).unwrap());
        }
        Ok(Tileset { sel_idx: 0, tiles })
    }

    pub fn get_idx(&self) -> u32 {
        self.sel_idx
    }

    // return true if vale changed
    pub fn set_idx(&mut self, new_idx: u32) -> bool {
        if new_idx < self.get_size() as u32 && new_idx != self.sel_idx {
            self.sel_idx = new_idx;
            return true;
        }
        false
    }

    pub fn get_size(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_valid_16(&self) -> bool {
        self.sel_idx + 16 + 1 < self.get_size() as u32
    }
}
