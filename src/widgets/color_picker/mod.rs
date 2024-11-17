mod imp;
pub mod operation;

use gtk::glib::{self, clone, signal::Propagation};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::data::palette::Palette;
use crate::widgets::window::Window;

use self::operation::ChangePaletteColor;

glib::wrapper! {
    pub struct ColorPicker(ObjectSubclass<imp::ColorPicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl ColorPicker {
    pub fn handle_action(&self, state: &Window) {
        let imp = self.imp();

        // undo triggers connect_value_changed, so pushing onto undo_stak can't be in the same
        // place
        imp.red_slider.connect_change_value(clone!(
            #[weak]
            state,
            #[upgrade_or]
            false.into(),
            move |_, _, val| {
                let val = (val.round() as u8).clamp(0, 31);
                let old_color = state.picker_color_inner();
                if val != old_color.red() {
                    // value changed, update undo queue
                    let new_color = old_color.with_red(val);
                    state.push_op(
                        ChangePaletteColor::new(state.palette_sel_idx(), old_color, new_color)
                            .into(),
                    );
                    return Propagation::Proceed;
                }
                // value not changed, block propogation
                return Propagation::Stop;
            }
        ));
        imp.red_slider.connect_value_changed(clone!(
            #[weak]
            state,
            move |this| {
                // change red value of picker color
                let val = this.adjustment().value();
                state.modify_picker_color(|c| c.with_red(val.round() as u8));
            }
        ));

        imp.green_slider.connect_change_value(clone!(
            #[weak]
            state,
            #[upgrade_or]
            false.into(),
            move |_, _, val| {
                let val = (val.round() as u8).clamp(0, 31);
                let old_color = state.picker_color_inner();
                if val != old_color.green() {
                    // value changed, update undo queue
                    let new_color = old_color.with_green(val);
                    state.push_op(
                        ChangePaletteColor::new(state.palette_sel_idx(), old_color, new_color)
                            .into(),
                    );
                    return Propagation::Proceed;
                }
                // value not changed, block propogation
                return Propagation::Stop;
            }
        ));
        imp.green_slider.connect_value_changed(clone!(
            #[weak]
            state,
            move |this| {
                // change green value of picker color
                let val = this.adjustment().value();
                state.modify_picker_color(|c| c.with_green(val.round() as u8));
            }
        ));

        imp.blue_slider.connect_change_value(clone!(
            #[weak]
            state,
            #[upgrade_or]
            false.into(),
            move |_, _, val| {
                let val = (val.round() as u8).clamp(0, 31);
                let old_color = state.picker_color_inner();
                if val != old_color.blue() {
                    // value changed, update undo queue
                    let new_color = old_color.with_blue(val);
                    state.push_op(
                        ChangePaletteColor::new(state.palette_sel_idx(), old_color, new_color)
                            .into(),
                    );
                    return Propagation::Proceed;
                }
                // value not changed, block propogation
                return Propagation::Stop;
            }
        ));
        imp.blue_slider.connect_value_changed(clone!(
            #[weak]
            state,
            move |this| {
                // change blue value of picker color
                let val = this.adjustment().value();
                state.modify_picker_color(|c| c.with_blue(val.round() as u8));
            }
        ));
    }

    pub fn render_widget(&self, state: &Window) {
        state.connect_picker_color_notify(clone!(
            #[weak(rename_to = this)]
            self,
            move |state| {
                let imp = this.imp();
                let color = state.picker_color_inner();

                // update slider position
                imp.red_adj.set_value(color.red().into());
                imp.green_adj.set_value(color.green().into());
                imp.blue_adj.set_value(color.blue().into());

                // redraw current color
                imp.color_square.queue_draw();
            }
        ));

        state.connect_palette_sel_idx_notify(move |state| {
            // load new color
            let Palette(palette_data): Palette = *state.palette_data();
            let new_color = palette_data[state.palette_sel_idx() as usize];
            if new_color == state.picker_color_inner() {
                return;
            }
            state.modify_picker_color(|_| new_color);
        });

        state.connect_palette_data_notify(move |state| {
            // load new color
            let Palette(palette_data): Palette = *state.palette_data();
            let new_color = palette_data[state.palette_sel_idx() as usize];
            if new_color == state.picker_color_inner() {
                return;
            }
            state.modify_picker_color(|_| new_color);
        });

        self.imp().color_square.set_draw_func(clone!(
            #[weak]
            state,
            move |_, cr, _, _| {
                let curr_color = state.picker_color_inner();
                let (r, g, b) = curr_color.to_cairo();
                cr.set_source_rgb(r, g, b);
                let _ = cr.paint();
            }
        ));
    }
}
