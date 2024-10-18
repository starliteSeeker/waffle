mod imp;

use std::path::PathBuf;
use std::str::FromStr;

use gio::{ActionEntry, SimpleActionGroup};
use glib::clone;
use gtk::GestureClick;
use gtk::{gio, glib};
use gtk::{prelude::*, subclass::prelude::*};

use strum::IntoEnumIterator;

use crate::data::{
    list_items::{Bpp, TileSize},
    tiles::RenameMeTileset,
};
use crate::utils::*;
use crate::widgets::window::Window;
use crate::TILE_W;

glib::wrapper! {
    pub struct TilePicker(ObjectSubclass<imp::TilePicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TilePicker {
    pub fn handle_action(&self, state: &Window) {
        let imp = self.imp();

        // mouse click on tileset drawing
        let gesture = GestureClick::new();
        gesture.connect_released(clone!(@weak self as this, @weak state => move |_, _, x, y| {
            // account for row offset when calculating correct idx
            let new_idx = (this.row_offset() as f64 + y / TILE_W) as u32 * 16 + (x / TILE_W) as u32;
            // TODO limit idx max
            if new_idx != state.tileset_sel_idx() {
                state.set_tileset_sel_idx(new_idx);
            }
        }));
        imp.tile_drawing.add_controller(gesture);

        // tile size dropdown
        imp.tile_size_select
            .connect_selected_notify(clone!(@weak imp, @weak state => move |_| {
                let new_size = TileSize::iter().nth(imp.tile_size_select.selected() as usize).unwrap();
                state.set_tile_size(new_size);
            }));

        // scroll up and down button
        let imp = self.imp();
        imp.tile_prev
            .connect_clicked(clone!(@weak self as this => move |_| {
                let x = this.row_offset();
                if x >= 8 {
                    this.set_row_offset(x - 8);
                }
            }));
        imp.tile_next
            .connect_clicked(clone!(@weak self as this, @weak state => move |_| {
                let x = this.row_offset();
                let max_tiles = state.tileset_data().0.len();
                if x + 8 + 8 < ((max_tiles + 15) / 16) as u32 {
                    this.set_row_offset(x - 8);
                }
            }));

        self.file_actions(state);
    }

    pub fn render_widget(&self, state: &Window) {
        state.connect_tileset_data_notify(clone!(@weak self as this => move |_| {
            this.imp().tile_drawing.queue_draw();
        }));

        state.connect_tileset_sel_idx_notify(clone!(@weak self as this => move |state| {
            this.set_index_label(state.tileset_sel_idx() as u16, state.tileset_data().0.len() as u16 - 1);
            this.imp().tile_drawing.queue_draw();
        }));

        self.connect_row_offset_notify(clone!(@weak self as this => move |_| {
            this.imp().tile_drawing.queue_draw();
        }));

        state.connect_palette_data_notify(clone!(@weak self as this => move |_| {
            this.imp().tile_drawing.queue_draw();
        }));

        state.connect_palette_sel_idx_notify(clone!(@weak self as this => move |_| {
            this.imp().tile_drawing.queue_draw();
        }));

        state.connect_tile_size_notify(clone!(@weak self as this => move |_| {
            this.imp().tile_drawing.queue_draw();
        }));

        self.imp().tile_drawing.set_draw_func(
            clone!(@weak self as this, @weak state => move |_, cr, w, _| {
                let tiles = &state.tileset_data();
                let row_offset = this.row_offset();

                // default color
                cr.set_source_rgb(0.4, 0.4, 0.4);
                let _ = cr.paint();

                let tile_w = w as f64 / 16.0;

                // 16 8x8 tiles per row
                for i in 0..256 {
                    let ti = (i + row_offset * 16) as usize;
                    if ti >= tiles.0.len() {
                        break;
                    }
                    let x_offset = (i % 16) as f64 * tile_w;
                    let y_offset = (i / 16) as f64 * tile_w;
                    let _ = cr.save();
                    cr.translate(x_offset, y_offset);
                    tiles.draw_tile(ti, cr, &state, None);
                    let _ = cr.restore();
                }

                // draw selected tile outline
                let _ = cr.save();
                cr.translate(0.0, -(row_offset as f64) * tile_w);
                let idx = state.tileset_sel_idx();
                let tile_size = state.tile_size();
                if state.is_valid_tileset_idx() {
                    cr.set_source_rgb(0.8, 0.8, 0.0);
                } else {
                    cr.set_source_rgb(0.5, 0.5, 0.5);
                }
                match tile_size {
                    TileSize::Eight => {
                        let x = (idx % 16) as f64 * tile_w;
                        let y = (idx / 16) as f64 * tile_w;
                        cr.rectangle(x, y, tile_w, tile_w);
                    },
                    TileSize::Sixteen => {
                        let x = (idx % 16) as f64 * tile_w;
                        let y = (idx / 16) as f64 * tile_w;
                        cr.rectangle(x, y, tile_w * 2.0, tile_w * 2.0);
                        if idx % 16 == 15 {
                            // wrap around
                            cr.rectangle(-tile_w, y + tile_w, tile_w * 2.0, tile_w * 2.0);
                        }
                    },
                };
                cr.clip_preserve();
                cr.set_line_width(2.0);
                let _ = cr.stroke();
                let _ = cr.restore();
            }),
        );
    }

    fn set_index_label(&self, idx: u16, max: u16) {
        self.imp()
            .tile_idx_label
            .set_label(&format!("${:03X} / ${:03X}", idx, max));
    }

    fn file_actions(&self, state: &Window) {
        let action_open = ActionEntry::builder("open")
            .parameter_type(Some(&String::static_variant_type()))
            .activate(
                clone!(@weak self as this, @weak state => move |_, _, parameter| {
                    // parse bit depth parameter
                    let Some(bpp) = parameter else {return};
                    let bpp = bpp.get::<String>().expect("parameter should have type String");
                    let bpp = Bpp::from_str(&bpp).expect("invalid bit depth");

                    file_open_dialog(state.clone(), move |path| {
                        match RenameMeTileset::from_file(&path, bpp) {
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                            Ok(t) => {
                                println!("load tileset: {path:?}");
                                state.set_tileset_data(t);
                                state.set_tileset_sel_idx(0);
                                this.set_row_offset(0);
                                state.set_tileset_file(Some(path));
                            }
                        }
                    });
                }),
            )
            .build();

        let action_reload = ActionEntry::builder("reload")
            .activate(clone!(@weak self as this, @weak state => move |_, _, _| {
                let Some(path) = state.tileset_file() else {
                    eprintln!("No palette file currently open");
                    return;
                };
                let bpp = state.tile_bpp();
                match RenameMeTileset::from_file(&path, bpp) {
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                    Ok(t) => {
                        println!("reload tileset: {path:?}");
                        state.set_tileset_data(t);
                        state.set_tileset_sel_idx(0);
                        this.set_row_offset(0);
                    }
                }
            }))
            .build();

        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_open, action_reload]);

        // bind file to reload action
        let reload = actions.lookup_action("reload").unwrap();
        state
            .bind_property("tileset_file", &reload, "enabled")
            .transform_to(|_, file: Option<PathBuf>| Some(file.is_some()))
            .sync_create()
            .build();

        state.insert_action_group("tiles", Some(&actions));
    }
}
