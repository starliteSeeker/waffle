use std::collections::HashMap;

use crate::data::tilemap::Tile;
use crate::undo_stack::UndoRedo;
use crate::widgets::window::Window;

pub struct ChangeTilemapTile {
    before: HashMap<(usize, usize), Tile>,
    after: Tile,
    was_dirty: bool,
}

impl UndoRedo for ChangeTilemapTile {
    fn undo(&self, state: &Window) {
        state.modify_tilemap_data(|tilemap| {
            for ((x, y), tile) in &self.before {
                tilemap.0[y * 32 + x] = *tile;
            }
            true
        });
        if state.tilemap_dirty() != self.was_dirty {
            state.set_tilemap_dirty(self.was_dirty);
        }
    }

    fn redo(&self, state: &Window) {
        state.modify_tilemap_data(|tilemap| {
            for (x, y) in self.before.keys() {
                tilemap.0[y * 32 + x] = self.after;
            }
            true
        });
        if !state.tilemap_dirty() {
            state.set_tilemap_dirty(true);
        }
    }
}

impl ChangeTilemapTile {
    pub fn new(before: HashMap<(usize, usize), Tile>, after: Tile, was_dirty: bool) -> Self {
        Self {
            before,
            after,
            was_dirty,
        }
    }
}
