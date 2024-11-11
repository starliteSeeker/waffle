use crate::data::color::Color;
use crate::undo_stack::UndoRedo;
use crate::widgets::window::Window;

pub struct ChangePaletteColor {
    pub idx: u8,
    before: Color,
    pub after: Color,
}

impl UndoRedo for ChangePaletteColor {
    fn undo(&self, state: &Window) {
        state.set_palette_sel_idx(self.idx);
        state.modify_picker_color(|_| self.before);
    }

    fn redo(&self, state: &Window) {
        state.set_palette_sel_idx(self.idx);
        state.modify_picker_color(|_| self.after);
    }
}

impl ChangePaletteColor {
    pub fn new(idx: u8, before: Color, after: Color) -> Self {
        ChangePaletteColor { idx, before, after }
    }
}
