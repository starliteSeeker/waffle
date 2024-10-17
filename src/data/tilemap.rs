use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::rc::Rc;

use modular_bitfield::prelude::*;

use crate::data::list_items::{BGMode, TileSize};
use crate::data::palette::Palette;
use crate::data::tiles::Tileset;

pub struct RenameMeTilemap(pub [Tile; 1024]);

impl Default for RenameMeTilemap {
    fn default() -> Self {
        RenameMeTilemap([Tile::default(); 1024])
    }
}

#[bitfield]
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Tile {
    pub tile_idx: B10,
    pub palette: B3,
    pub priority: bool,
    pub x_flip: bool,
    pub y_flip: bool,
}

pub struct Tilemap {
    tiles: Vec<Tile>,
}

impl Default for Tilemap {
    fn default() -> Self {
        let arr = vec![Tile::new(); 32 * 32];
        Tilemap { tiles: arr }
    }
}

impl Tilemap {
    pub fn from_path(path: &std::path::PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read(&path)?;
        let len = content.len();
        // check alignment
        if len % 2 != 0 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("file size is {} but should be a multiple of 2", len),
            ));
        }
        // check file size
        if len > 32 * 32 * 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "file must not exceed 2048 bytes",
            ));
        }

        Ok(Tilemap {
            tiles: {
                let mut v = Vec::with_capacity(len / 2);
                for i in (0..len).step_by(2) {
                    v.push(Tile::from_bytes([content[i], content[i + 1]]));
                }
                v
            },
        })
    }

    pub fn write_to_file(&self, mut f: &File) -> std::io::Result<()> {
        for c in &self.tiles {
            f.write_all(&c.into_bytes())?;
        }
        Ok(())
    }

    // return true if new tile is different from old one
    pub fn set_tile(&mut self, idx: u32, tile: &Tile) -> bool {
        if idx >= self.tiles.len() as u32 {
            return false;
        }
        if self.tiles[idx as usize] == *tile {
            return false;
        }

        self.tiles[idx as usize] = *tile;
        return true;
    }

    pub fn draw(
        &self,
        cr: &gtk::cairo::Context,
        palette_data: Rc<RefCell<Palette>>,
        tile_data: Rc<RefCell<Tileset>>,
        bg_mode: Rc<RefCell<BGMode>>,
        tile_size: Rc<RefCell<TileSize>>,
        curr_drag: Option<((u32, u32), (u32, u32))>,
        curr_tile: Tile,
    ) {
        // fallback color
        cr.set_source_rgb(0.4, 0.4, 0.4);
        let _ = cr.paint();

        let bg_mode = bg_mode.borrow();

        // sort rectangle fill boundaries
        let ((x_min, x_max), (y_min, y_max)) = if let Some(((x1, y1), (x2, y2))) = curr_drag {
            (
                (x1.min(x2) as usize, x1.max(x2) as usize),
                (y1.min(y2) as usize, y1.max(y2) as usize),
            )
        } else {
            ((1, 0), (1, 0)) // impossible arrangement, range check will return false
        };

        for (i, tile) in self.tiles.iter().enumerate() {
            let ix = i % 32;
            let iy = i / 32;
            let x_offset = ix as f64 * crate::TILE_W;
            let y_offset = iy as f64 * crate::TILE_W;

            // draw ractangle fill graphics if index is in range
            let tile = if ix >= x_min && ix <= x_max && iy >= y_min && iy <= y_max {
                curr_tile
            } else {
                *tile
            };

            let _ = cr.save();
            cr.translate(x_offset, y_offset);
            if tile.x_flip() {
                cr.translate(crate::TILE_W, 0.0);
                cr.scale(-1.0, 1.0);
            }
            if tile.y_flip() {
                cr.translate(0.0, crate::TILE_W);
                cr.scale(1.0, -1.0);
            }

            tile_data.borrow().draw_tile_size(
                tile.tile_idx(),
                cr,
                palette_data.clone(),
                Some(bg_mode.palette_offset() + bg_mode.bpp().to_val() * tile.palette()),
                &bg_mode,
                &tile_size.borrow(),
            );
            let _ = cr.restore();
        }
    }
}
