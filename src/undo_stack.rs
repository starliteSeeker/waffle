use std::cell::OnceCell;
use std::collections::VecDeque;

use crate::widgets::window::Window;
use crate::widgets::{
    color_picker::operation::ChangePaletteColor, tilemap_editor::operation::ChangeTilemapTile,
};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait UndoRedo {
    fn undo(&self, state: &Window);
    fn redo(&self, state: &Window);
}

#[enum_dispatch(UndoRedo)]
pub enum Operation {
    ChangePaletteColor,
    ChangeTilemapTile,
}

#[derive(Default)]
pub struct UndoStack {
    state: OnceCell<Window>,
    stack: VecDeque<Operation>,
    curr: usize,                  // current position in stack
    palette_dirty: Option<isize>, // number of operations away from a clean copy of palette data
    tilemap_dirty: Option<isize>, // number of operations away from a clean copy of tilemap data
}

impl UndoStack {
    // max stack size
    const MAX: usize = usize::MAX / 4;

    pub fn init(&mut self, state: Window) {
        if self.state.set(state).is_err() {
            eprintln!("already init");
        }
        self.palette_dirty = Some(0);
        self.tilemap_dirty = Some(0);
    }

    pub fn clear(&mut self) {
        self.stack.clear();
        self.curr = 0;
    }

    pub fn push(&mut self, op: Operation) {
        // empty stack entry after curr
        self.stack.truncate(self.curr);
        if self.palette_dirty.is_some_and(|n| n < 0) {
            self.palette_dirty = None;
        }
        if self.tilemap_dirty.is_some_and(|n| n < 0) {
            self.tilemap_dirty = None;
        }

        // combine with previous operation, or push directly onto stack
        let palette_dirty = self.palette_dirty();
        match (self.stack.back_mut(), &op) {
            // 2 consecutive change color op without a save in between
            (Some(Operation::ChangePaletteColor(old)), Operation::ChangePaletteColor(new))
                if old.idx == new.idx && palette_dirty =>
            {
                old.after = new.after;
            }
            (_, new) => {
                match new {
                    Operation::ChangePaletteColor(_) => {
                        self.palette_dirty = self.palette_dirty.map(|n| n + 1)
                    }
                    Operation::ChangeTilemapTile(_) => {
                        self.tilemap_dirty = self.tilemap_dirty.map(|n| n + 1)
                    }
                }
                // limit stack size to Self::MAX
                if self.stack.len() >= Self::MAX {
                    self.stack.pop_front();
                } else {
                    self.curr += 1;
                }
                self.stack.push_back(op);
            }
        }
    }

    pub fn undo(&mut self) {
        if self.curr <= 0 {
            return;
        }
        let Some(op) = self.stack.get(self.curr - 1) else {
            return;
        };
        let Some(state) = self.state.get() else {
            return;
        };
        match op {
            Operation::ChangePaletteColor(_) => {
                self.palette_dirty = self.palette_dirty.map(|n| n - 1)
            }
            Operation::ChangeTilemapTile(_) => {
                self.tilemap_dirty = self.tilemap_dirty.map(|n| n - 1)
            }
        }
        self.curr -= 1;
        op.undo(state);
    }

    pub fn redo(&mut self) {
        if self.curr + 1 > self.stack.len() {
            return;
        }
        let Some(op) = self.stack.get(self.curr) else {
            return;
        };
        let Some(state) = self.state.get() else {
            return;
        };
        match op {
            Operation::ChangePaletteColor(_) => {
                self.palette_dirty = self.palette_dirty.map(|n| n + 1)
            }
            Operation::ChangeTilemapTile(_) => {
                self.tilemap_dirty = self.tilemap_dirty.map(|n| n + 1)
            }
        }
        self.curr += 1;
        op.redo(state);
    }

    pub fn palette_dirty(&self) -> bool {
        !self.palette_dirty.is_some_and(|n| n == 0)
    }
    pub fn mark_palette_clean(&mut self) {
        self.palette_dirty = Some(0);
    }

    pub fn tilemap_dirty(&self) -> bool {
        !self.tilemap_dirty.is_some_and(|n| n == 0)
    }
    pub fn mark_tilemap_clean(&mut self) {
        self.tilemap_dirty = Some(0);
    }
}
