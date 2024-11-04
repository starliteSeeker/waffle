use std::cell::OnceCell;

use crate::widgets::window::Window;
use crate::widgets::{
    color_picker::operation::ChangePaletteColor, tilemap_editor::operation::ChangeTilemapTile,
};
use enum_dispatch::enum_dispatch;

#[enum_dispatch]
pub trait UndoRedo {
    fn undo(&self, state: &Window);
}

#[enum_dispatch(UndoRedo)]
pub enum Operation {
    ChangePaletteColor,
    ChangeTilemapTile,
}

#[derive(Default)]
pub struct UndoStack {
    state: OnceCell<Window>,
    stack: Vec<Operation>,
}

impl UndoStack {
    pub fn init(&mut self, state: Window) {
        if self.state.set(state).is_err() {
            eprintln!("already init");
        }
    }
    pub fn push(&mut self, op: Operation) {
        // combine with previous operation, or push directly onto stack
        match (self.stack.last_mut(), &op) {
            (Some(Operation::ChangePaletteColor(old)), Operation::ChangePaletteColor(new))
                if old.idx == new.idx =>
            {
                old.after = new.after;
            }
            _ => {
                self.stack.push(op);
            }
        }
    }

    pub fn undo(&mut self) {
        let Some(op) = self.stack.pop() else { return };
        let Some(state) = self.state.get() else {
            return;
        };
        op.undo(state);
    }
}
