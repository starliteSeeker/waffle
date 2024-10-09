mod imp;

use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::data::palette::RenameMePalette;
use crate::widgets::window::Window;

glib::wrapper! {
    pub struct ColorPicker(ObjectSubclass<imp::ColorPicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl ColorPicker {
    pub fn handle_action(&self, state: &Window) {
        // rgb sliders
        let imp = self.imp();
        imp.red_slider
            .connect_value_changed(clone!(@weak state => move |this| {
                // change red value of picker color
                let val = this.adjustment().value();
                state.modify_picker_color(|c| c.with_red(val.round() as u8));
            }));
        imp.green_slider
            .connect_value_changed(clone!(@weak state => move |this| {
                // change green value of picker color
                let val = this.adjustment().value();
                state.modify_picker_color(|c| c.with_green(val.round() as u8));
            }));
        imp.blue_slider
            .connect_value_changed(clone!(@weak state => move |this| {
                // change blue value of picker color
                let val = this.adjustment().value();
                state.modify_picker_color(|c| c.with_blue(val.round() as u8));
            }));
    }

    pub fn render_widget(&self, state: &Window) {
        state.connect_picker_color_notify(clone!(@weak self as this => move |state| {
            let imp = this.imp();
            let color = state.picker_color_inner();

            // update slider position
            imp.red_adj.set_value(color.red().into());
            imp.green_adj.set_value(color.green().into());
            imp.blue_adj.set_value(color.blue().into());

            // redraw current color
            imp.color_square.queue_draw();
        }));

        state.connect_palette_sel_idx_notify(clone!(@weak self as this => move |state| {
            // load new color
            let RenameMePalette(palette_data): RenameMePalette = *state.palette_data().unwrap().borrow();
            let new_color = palette_data[state.palette_sel_idx() as usize];
            if new_color == state.picker_color_inner() {
                return;
            }
            state.modify_picker_color(|_| new_color);
        }));

        self.imp()
            .color_square
            .set_draw_func(clone!(@weak state => move |_, cr, _, _| {
                let curr_color = state.picker_color_inner();
                let (r, g, b) = curr_color.to_cairo();
                cr.set_source_rgb(r, g, b);
                let _ = cr.paint();
            }));
    }
}
