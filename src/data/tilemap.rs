use std::cell::RefCell;
use std::fs::File;
use std::io::Write;
use std::mem::{self, MaybeUninit};
use std::rc::Rc;

use itertools::Itertools;
use modular_bitfield::prelude::*;

use crate::data::list_items::{BGMode, TileSize};
use crate::data::palette::Palette;
use crate::data::tiles::Tileset;

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
    tiles: [Tile; 32 * 32],
}

impl Default for Tilemap {
    fn default() -> Self {
        let arr = [Tile::new(); 32 * 32];
        Tilemap { tiles: arr }
    }
}

impl Tilemap {
    pub fn from_path(path: &std::path::PathBuf) -> std::io::Result<Self> {
        let content = std::fs::read(&path)?;
        let len = content.len();
        if len != 32 * 32 * 2 {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "file must be 2048 bytes",
            ));
        }

        // unsafe initializing array
        // https://doc.rust-lang.org/core/mem/union.MaybeUninit.html#initializing-an-array-element-by-element
        Ok(Tilemap {
            tiles: {
                let mut data: [MaybeUninit<Tile>; 32 * 32] =
                    unsafe { MaybeUninit::uninit().assume_init() };

                for (i, (lo, hi)) in content.into_iter().tuples().enumerate() {
                    data[i].write(Tile::from_bytes([lo, hi]));
                }

                unsafe { mem::transmute::<_, [Tile; 32 * 32]>(data) }
            },
        })
    }

    pub fn write_to_file(&self, mut f: &File) -> std::io::Result<()> {
        for c in self.tiles {
            f.write_all(&c.into_bytes())?;
        }
        Ok(())
    }

    // return true if new tile is different from old one
    pub fn set_tile(&mut self, idx: u32, tile: &Tile) -> bool {
        if idx >= 32 * 32 {
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
    ) {
        // default color
        cr.set_source_rgb(0.0, 1.0, 1.0);
        let _ = cr.paint();

        let bg_mode = bg_mode.borrow();

        for (i, tile) in self.tiles.into_iter().enumerate() {
            let x_offset = (i % 32) as f64 * crate::TILE_W;
            let y_offset = (i / 32) as f64 * crate::TILE_W;

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
