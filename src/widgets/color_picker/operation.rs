use crate::data::color::Color;
use crate::undo_stack::UndoRedo;
use crate::widgets::window::Window;

pub struct ChangePaletteColor {
    pub idx: u8,
    before: Color,
    pub after: Color,
    was_dirty: bool,
}

impl UndoRedo for ChangePaletteColor {
    fn undo(&self, state: &Window) {
        state.set_palette_sel_idx(self.idx);
        state.modify_picker_color(|_| self.before);
        if state.palette_dirty() != self.was_dirty {
            state.set_palette_dirty(self.was_dirty);
        }
    }

    fn redo(&self, state: &Window) {
        state.set_palette_sel_idx(self.idx);
        state.modify_picker_color(|_| self.after);
        if !state.palette_dirty() {
            state.set_palette_dirty(true);
        }
    }
}

impl ChangePaletteColor {
    pub fn new(idx: u8, before: Color, after: Color, was_dirty: bool) -> Self {
        ChangePaletteColor {
            idx,
            before,
            after,
            was_dirty,
        }
    }
}
