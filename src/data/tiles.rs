use crate::data::palette::Palette;
use std::cell::RefCell;
use std::rc::Rc;

use crate::data::list_items::{BGMode, Bpp, TileSize};
use crate::TILE_W;

pub struct Tile {
    chr: [u8; 64],
    bpp: Bpp,
}

impl Tile {
    fn from_2bpp(s: &[u8]) -> Option<Self> {
        // 8 * 8 pixels * 2 (bits/pixel) / 8 (bits/byte)
        if s.len() != 16 {
            return None;
        }

        // s = [bit 0 of pixels 0-7, bit 1 of pixels 0-7,
        //      bit 0 of pixels 8-15, bit 1 of pixels 8-15, ...]
        let mut chr = [0; 64];
        for i in 0..64 {
            let a = i / 8;
            let b = i % 8;
            chr[i] = (s[2 * a] >> (7 - b)) & 0b1; // bit 0
            chr[i] |= ((s[2 * a + 1] >> (7 - b)) & 0b1) << 1; // bit 1
        }
        Some(Tile { chr, bpp: Bpp::Two })
    }

    fn from_4bpp(s: &[u8]) -> Option<Self> {
        // 8 * 8 pixels * 4 (bits/pixel) / 8 (bits/byte)
        if s.len() != 32 {
            return None;
        }

        // s = [bit 0 of pixels 0-7, bit 1 of pixels 0-7,
        //      bit 0 of pixels 8-15, bit 1 of pixels 8-15,
        //      ...
        //      bit 0 of pixels 56-63, bit 1 of pixels 56-63,
        //      bit 2 of pixels 0-7, bit 3 of pixels 0-7,
        //      ...
        //      bit 2 of pixels 56-63, bit 1 of pixels 56-63]
        let mut chr = [0; 64];
        for i in 0..64 {
            let a = i / 8;
            let b = i % 8;
            chr[i] = (s[2 * a] >> (7 - b)) & 0b1; // bit 0
            chr[i] |= ((s[2 * a + 1] >> (7 - b)) & 0b1) << 1; // bit 1
            chr[i] |= ((s[16 + 2 * a] >> (7 - b)) & 0b1) << 2; // bit 2
            chr[i] |= ((s[16 + 2 * a + 1] >> (7 - b)) & 0b1) << 3; // bit 3
        }
        Some(Tile {
            chr,
            bpp: Bpp::Four,
        })
    }

    pub fn draw(
        &self,
        cr: &gtk::cairo::Context,
        palette_data: Rc<RefCell<Palette>>,
        palette_offset: Option<u8>,
        bg_mode: &BGMode,
    ) {
        let pxl_w = TILE_W / 8.0;
        // collect pixels with same color, then draw the pixels together
        let mut rects = vec![Vec::new(); self.bpp.to_val() as usize];

        // (0, 0) as top left corner of tile
        for (j, c) in self.chr.into_iter().enumerate() {
            // top left corner of pixel
            let x_off = (j % 8) as f64 * pxl_w;
            let y_off = (j / 8) as f64 * pxl_w;
            // fail silently if c is out of range (>=4 for 2bpp, >=16 for 4bpp)
            rects.get_mut(c as usize).map(|v| v.push((x_off, y_off)));
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
    sel_idx: u16,
    pub tiles: Vec<Tile>,
    bpp: Bpp,
}

impl Default for Tileset {
    fn default() -> Self {
        Tileset {
            sel_idx: 0,
            tiles: vec![Tile {
                chr: [0; 64],
                bpp: Bpp::Two,
            }],
            bpp: Bpp::Two,
        }
    }
}

impl Tileset {
    // tile index is stored as 10-bit integer in tilemap::Tile
    const MAX: usize = 0b1 << 10;

    pub fn from_path(path: &std::path::PathBuf, bpp: Bpp) -> std::io::Result<Self> {
        let content = std::fs::read(path)?;
        let len = content.len();
        if len == 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "file has length 0",
            ));
        }
        let align = 8 * 8 * bpp.bits() as usize / 8;
        if len % align != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("file does not align with {align} bytes"),
            ));
        }

        if len / align > Self::MAX {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("tile count exceeds maximum of {} tiles", Self::MAX),
            ));
        }

        let mut tiles = Vec::new();
        for i in (0..len).step_by(align) {
            let tile = match bpp {
                Bpp::Two => Tile::from_2bpp(&content[i..i + align]).unwrap(),
                Bpp::Four => Tile::from_4bpp(&content[i..i + align]).unwrap(),
            };
            tiles.push(tile);
        }
        Ok(Tileset {
            sel_idx: 0,
            tiles,
            bpp,
        })
    }

    pub fn get_idx(&self) -> u16 {
        self.sel_idx
    }

    // return true if value changed
    pub fn set_idx(&mut self, new_idx: u16) -> bool {
        if new_idx < self.get_size() as u16 && new_idx != self.sel_idx {
            self.sel_idx = new_idx;
            return true;
        }
        false
    }

    pub fn bpp(&self) -> Bpp {
        self.bpp
    }

    pub fn get_size(&self) -> usize {
        self.tiles.len()
    }

    pub fn is_valid_16(&self) -> bool {
        self.sel_idx + 16 + 1 < self.get_size() as u16
    }

    // draw an 8x8 tile, or an invalid tile if index is out of range
    pub fn draw_tile(
        &self,
        idx: u16,
        cr: &gtk::cairo::Context,
        palette_data: Rc<RefCell<Palette>>,
        palette_offset: Option<u8>,
        bg_mode: &BGMode,
    ) {
        match self.tiles.get(idx as usize) {
            Some(tile) => {
                tile.draw(cr, palette_data, palette_offset, bg_mode);
            }
            None => {
                // pink tile with dot at the center
                let _ = cr.save();
                cr.rectangle(0.0, 0.0, TILE_W, TILE_W);
                cr.set_source_rgb(1.0, 0.8, 0.8);
                let _ = cr.fill();
                cr.arc(
                    TILE_W / 2.0,
                    TILE_W / 2.0,
                    TILE_W / 6.0,
                    0.0,
                    2.0 * std::f64::consts::PI,
                );
                cr.set_source_rgb(1.0, 0.7, 0.7);
                let _ = cr.fill();
                let _ = cr.restore();
            }
        }
    }

    // draw 8x8 or 16x16 tile depending on tile_size
    pub fn draw_tile_size(
        &self,
        idx: u16,
        cr: &gtk::cairo::Context,
        palette_data: Rc<RefCell<Palette>>,
        palette_offset: Option<u8>,
        bg_mode: &BGMode,
        tile_size: &TileSize,
    ) {
        match tile_size {
            TileSize::Eight => {
                self.draw_tile(idx, cr, palette_data.clone(), palette_offset, &bg_mode);
            }
            TileSize::Sixteen => {
                // (idx   )(idx+ 1)
                // (idx+16)(idx+17)
                let _ = cr.save();
                cr.scale(0.5, 0.5);
                self.draw_tile(idx, cr, palette_data.clone(), palette_offset, &bg_mode);
                cr.translate(TILE_W, 0.0);
                self.draw_tile(idx + 1, cr, palette_data.clone(), palette_offset, &bg_mode);
                cr.translate(0.0, TILE_W);
                self.draw_tile(idx + 17, cr, palette_data.clone(), palette_offset, &bg_mode);
                cr.translate(-TILE_W, 0.0);
                self.draw_tile(idx + 16, cr, palette_data.clone(), palette_offset, &bg_mode);
                let _ = cr.restore();
            }
        }
    }
}
