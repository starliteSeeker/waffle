use std::cell::RefCell;
use std::rc::Rc;

use crate::data::palette::Palette;
use crate::data::tiles::Tileset;

#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub struct Tile {
    pub tile_idx: u32, // 10 bit
    pub x_flip: bool,
    pub y_flip: bool,
    pub palette: u8, // 0-7
}

pub struct Tilemap {
    tiles: [Tile; 32 * 32],
}

impl Default for Tilemap {
    fn default() -> Self {
        let mut arr = [Default::default(); 32 * 32];
        arr[0] = Tile {
            tile_idx: 1,
            x_flip: false,
            y_flip: false,
            palette: 1,
        };
        Tilemap { tiles: arr }
    }
}

impl Tilemap {
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
    ) {
        // default color
        cr.set_source_rgb(0.0, 1.0, 1.0);
        let _ = cr.paint();

        for (i, tile) in self.tiles.into_iter().enumerate() {
            let x_offset = (i % 32) as f64 * crate::TILE_W;
            let y_offset = (i / 32) as f64 * crate::TILE_W;

            let _ = cr.save();
            cr.translate(x_offset, y_offset);
            if tile.x_flip {
                cr.translate(crate::TILE_W, 0.0);
                cr.scale(-1.0, 1.0);
            }
            if tile.y_flip {
                cr.translate(0.0, crate::TILE_W);
                cr.scale(1.0, -1.0);
            }
            tile_data.borrow().tiles[tile.tile_idx as usize].draw(
                cr,
                palette_data.clone(),
                Some(tile.palette),
            );
            let _ = cr.restore();
        }
    }
}
