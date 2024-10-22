use crate::data::list_items::Bpp;
use crate::widgets::window::Window;
use crate::TILE_W;

pub struct TileData(pub [u8; 64]);

impl Default for TileData {
    fn default() -> Self {
        Self([0; 64])
    }
}

impl TileData {
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
        Some(Self(chr))
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
        Some(Self(chr))
    }

    fn draw(&self, cr: &gtk::cairo::Context, state: &Window, palette_subset: Option<u8>) {
        let pxl_w = TILE_W / 8.0;
        // collect pixels with same color, then draw the pixels together
        let mut rects = vec![Vec::new(); state.tile_bpp().to_val() as usize];

        // (0, 0) as top left corner of tile
        for (j, c) in self.0.into_iter().enumerate() {
            // top left corner of pixel
            let x_off = (j % 8) as f64 * pxl_w;
            let y_off = (j / 8) as f64 * pxl_w;
            // fail silently if c is out of range (>=4 for 2bpp, >=16 for 4bpp)
            rects.get_mut(c as usize).map(|v| v.push((x_off, y_off)));
        }

        let palette_data = state.palette_data();
        let color_zero_idx = if let Some(s) = palette_subset {
            state.palette_base() + s * state.tile_bpp().to_val()
        } else {
            state.palette_sel_idx() - state.palette_sel_idx() % state.tile_bpp().to_val()
        };

        for (i, v) in rects.into_iter().enumerate() {
            for (x, y) in v {
                cr.rectangle(x, y, pxl_w, pxl_w);
            }
            let (r, g, b) = palette_data.0[color_zero_idx as usize + i].to_cairo();
            cr.set_source_rgb(r, g, b);
            let _ = cr.fill();
        }
    }
}

pub struct Tileset(pub Vec<TileData>);

impl Default for Tileset {
    fn default() -> Self {
        Self(vec![TileData::default()])
    }
}

impl Tileset {
    const MAX: usize = 0b1 << 10;

    pub fn from_file(path: &std::path::PathBuf, bpp: Bpp) -> std::io::Result<Self> {
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
                Bpp::Two => TileData::from_2bpp(&content[i..i + align]).unwrap(),
                Bpp::Four => TileData::from_4bpp(&content[i..i + align]).unwrap(),
            };
            tiles.push(tile);
        }
        Ok(Self(tiles))
    }

    pub fn draw_tile(
        &self,
        idx: usize,
        cr: &gtk::cairo::Context,
        state: &Window,
        palette_subset: Option<u8>,
    ) {
        if let Some(tile) = self.0.get(idx) {
            tile.draw(cr, state, palette_subset);
        } else {
            // pink tile with dot at the center
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
        }
    }
}
