mod imp;

/*
use std::cell::RefCell;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use std::str::FromStr;

use std::fs::File;
*/

// use gio::{ActionEntry, SimpleActionGroup};
use glib::{clone, closure_local};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::GestureClick;
use gtk::{gio, glib};

// use crate::data::file_format::PaletteFile;
use crate::data::list_items::Bpp;
use crate::data::palette::RenameMePalette;
// use crate::utils::*;
use crate::widgets::window::Window;
use crate::TILE_W;

glib::wrapper! {
    pub struct PalettePicker(ObjectSubclass<imp::PalettePicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl PalettePicker {
    pub fn handle_action(&self, state: &Window) {
        // mouse click on palette
        let gesture = GestureClick::new();
        gesture.connect_released(
            clone!(@weak self as this , @weak state => move |_, _, x, y| {
                // account for y scroll when calculating correct idx
                let yy = y + this.imp().palette_scroll.vadjustment().value();
                let new_idx = (yy / TILE_W) as u8 * 16 + (x / TILE_W) as u8;
                if new_idx != state.palette_sel_idx() {
                    state.set_palette_sel_idx(new_idx);
                }
            }),
        );
        self.imp().palette_scroll.add_controller(gesture);
    }

    pub fn render_widget(&self, state: &Window) {
        state.connect_palette_sel_idx_notify(clone!(@weak self as this => move |state| {
            this.imp().palette_drawing.queue_draw();
            this.set_label(state.palette_sel_idx());
        }));

        state.connect_palette_data_notify(clone!(@weak self as this => move |_| {
            this.imp().palette_drawing.queue_draw();
        }));
        state.connect_tile_bpp_notify(clone!(@weak self as this => move |_| {
            this.imp().palette_drawing.queue_draw();
        }));

        state.connect_picker_color_notify(move |state| {
            let idx = state.palette_sel_idx() as usize;
            let new_color = state.picker_color_inner();
            state.modify_palette_data(|RenameMePalette(palette)| {
                if palette[idx] != new_color {
                    palette[idx] = new_color;
                    return true;
                } else {
                    return false;
                }
            })
        });

        self.imp()
            .palette_drawing
            .set_draw_func(clone!(@weak state => move |_, cr, x, y| {
                let RenameMePalette(palette_data): RenameMePalette = *state.palette_data().unwrap().borrow();
                let sel_idx = state.palette_sel_idx();
                let tile_bpp = state.tile_bpp();
                let bg_mode = state.bg_mode();

                // default color
                cr.set_source_rgb(1.0, 0.0, 1.0);
                let _ = cr.paint();

                // draw palette
                cr.set_line_width(1.0);
                for i in 0..16 {
                    for j in 0..16 {
                        let x_offset = j as f64 * TILE_W;
                        let y_offset = i as f64 * TILE_W;
                        cr.rectangle(x_offset, y_offset, TILE_W, TILE_W);
                        let (red, green, blue) = palette_data[i * 16 + j].to_cairo();
                        cr.set_source_rgb(red, green, blue);
                        let _ = cr.fill();

                        // add marker for transparent palette color
                        let is_transparent = match tile_bpp {
                            Bpp::Two => j % 4 == 0,
                            Bpp::Four => j == 0,
                        };
                        if is_transparent {
                            cr.arc(x_offset + TILE_W / 2.0, y_offset + TILE_W / 2.0, 3.0, 0.0 , 2.0 * std::f64::consts::PI);
                            cr.set_source_rgb(0.8, 0.8, 0.8);
                            let _ = cr.fill_preserve();
                            cr.set_source_rgb(0.0, 0.0, 0.0);
                            let _ = cr.stroke();
                        }
                    }
                }

                // dim sections of palette not used in current bg_mode
                let x_offset = 0.0;
                let y_offset = (state.palette_base() / 16) as f64 * TILE_W;
                let width = 16.0 * TILE_W;
                let height = (tile_bpp.to_val() * 8 / 16) as f64 * TILE_W;
                cr.rectangle(x_offset, y_offset, width, height);
                cr.rectangle(0.0, 0.0, x as f64, y as f64);
                let _ = cr.save();
                cr.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.7);
                let _ = cr.fill();
                let _ = cr.restore();

                // draw current palette group outline
                let pal_start_idx = sel_idx - (sel_idx % tile_bpp.to_val());
                let x_offset = (pal_start_idx % 16) as f64 * TILE_W;
                let y_offset = (pal_start_idx / 16) as f64 * TILE_W;
                cr.rectangle(x_offset, y_offset, TILE_W * tile_bpp.to_val() as f64, TILE_W);

                cr.clip_preserve();
                cr.set_line_width(2.0);
                cr.set_source_rgb(0.8, 0.8, 0.0);
                let _ = cr.stroke();

                // draw current color outline
                let x_offset = (sel_idx % 16) as f64 * TILE_W;
                let y_offset = (sel_idx / 16) as f64 * TILE_W;
                cr.rectangle(x_offset, y_offset, TILE_W, TILE_W);
                cr.clip_preserve();
                cr.set_line_width(4.0);
                cr.set_source_rgb(1.0, 1.0, 0.0);
                let _ = cr.stroke();
            }));
    }

    fn set_label(&self, idx: u8) {
        self.imp()
            .color_idx_label
            .set_label(&format!("${:02X} / $FF", idx));
    }
}
