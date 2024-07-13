mod imp;

use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use gio::{ActionEntry, SimpleActionGroup};
use glib::{clone, closure_local};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::GestureClick;
use gtk::{gio, glib};

use crate::data::list_items::{BGMode, TileSize};
use crate::data::palette::Palette;
use crate::data::tiles::Tileset;
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
    pub fn setup_all<P: WidgetExt, M: WidgetExt>(
        &self,
        parent: Window,
        pal: Rc<RefCell<Palette>>,
        tile_data: Rc<RefCell<Tileset>>,
        palette_obj: P,
        bg_mode: Rc<RefCell<BGMode>>,
        map_obj: M,
    ) {
        self.setup_gesture(tile_data.clone());
        self.setup_actions(parent, tile_data.clone());
        self.setup_signal_connection(tile_data.clone(), palette_obj.clone(), map_obj);
        self.setup_draw(pal, tile_data, bg_mode);
    }

    fn setup_gesture(&self, tile_data: Rc<RefCell<Tileset>>) {
        let gesture = GestureClick::new();
        gesture.connect_released(clone!(@weak self as this => move |_, _, x, y| {
            // account for row offset when calculating correct idx
            let new_idx = (*this.imp().row_offset.borrow() as f64 + y / TILE_W) as u16 * 16 + (x / TILE_W) as u16;
            // emit signal
            if tile_data.borrow_mut().set_idx(new_idx) {
                this.emit_by_name::<()>("tile-idx-changed", &[&(new_idx as u32)]);
            }
        }));
        self.imp().tile_drawing.add_controller(gesture);
    }

    fn setup_actions(&self, parent: Window, tile_data: Rc<RefCell<Tileset>>) {
        let action_open = ActionEntry::builder("open")
            .activate(
                clone!(@weak self as this, @weak tile_data, @weak parent => move |_, _, _| {
                    file_open_dialog(parent, move |path| {
                        match Tileset::from_path(&path) {
                            Err(e) => {
                                eprintln!("Error: {}", e);
                            }
                            Ok(t) => {
                                println!("load tileset: {path:?}");
                                *tile_data.borrow_mut() = t;
                                *this.imp().row_offset.borrow_mut() = 0;
                                this.set_file(Some(path));
                                this.emit_by_name::<()>("tile-changed", &[]);
                            }
                        }
                    });
                }),
            )
            .build();

        let action_reload = ActionEntry::builder("reload")
            .activate(
                clone!(@weak self as this, @weak tile_data => move |_, _, _| {
                    let Some(path) = this.file() else {
                        eprintln!("No palette file currently open");
                        return;
                    };
                    match Tileset::from_path(&path) {
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                        Ok(t) => {
                            println!("load tileset: {path:?}");
                            *tile_data.borrow_mut() = t;
                            *this.imp().row_offset.borrow_mut() = 0;
                            this.emit_by_name::<()>("tile-changed", &[]);
                        }
                    }
                }),
            )
            .build();
        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_open, action_reload]);

        // bind file to reload action
        let reload = actions.lookup_action("reload").unwrap();
        self.bind_property("file", &reload, "enabled")
            .transform_to(|_, file: Option<PathBuf>| Some(file.is_some()))
            .sync_create()
            .build();

        parent.insert_action_group("tiles", Some(&actions));
    }

    fn setup_signal_connection<P: WidgetExt, M: WidgetExt>(
        &self,
        tile_data: Rc<RefCell<Tileset>>,
        palette_obj: P,
        map_obj: M,
    ) {
        // redraw self
        self.connect_closure(
            "tile-idx-changed",
            false,
            closure_local!(|this: Self, _: u32| {
                this.imp().tile_drawing.queue_draw();
            }),
        );
        self.connect_closure(
            "tile-changed",
            false,
            closure_local!(|this: Self| {
                this.imp().tile_drawing.queue_draw();
            }),
        );
        palette_obj.connect_closure(
            "palette-idx-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: P, _: u8| {
                let Some(this) = this else {return};
                this.imp().tile_drawing.queue_draw();
            }),
        );
        palette_obj.connect_closure(
            "palette-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: P| {
                let Some(this) = this else {return};
                this.imp().tile_drawing.queue_draw();
            }),
        );

        map_obj.connect_closure(
            "bg-mode-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: M| {
                let Some(this) = this else {return};
                this.imp().tile_drawing.queue_draw();
            }),
        );

        // update tile index label
        self.connect_closure(
            "tile-idx-changed",
            false,
            closure_local!(@weak-allow-none tile_data => move |this: Self, _: u32| {
                let Some(tile_data) = tile_data else {return};
                this.imp().tile_idx_label.set_label(&format!("${:03X} / ${:03X}", tile_data.borrow().get_idx(), tile_data.borrow().get_size() - 1));
            }),
        );

        // scroll up and down button
        let imp = self.imp();
        imp.tile_prev
            .connect_clicked(clone!(@weak imp, @weak tile_data => move |_| {
                let x = *imp.row_offset.borrow();
                if x >= 8 {
                    *imp.row_offset.borrow_mut() = x - 8;
                }
                imp.tile_drawing.queue_draw();
            }));
        imp.tile_next.connect_clicked(clone!(@weak imp => move |_| {
            let x = *imp.row_offset.borrow();
            let max_row = (tile_data.borrow().get_size() + 15) / 16;
            if x + 8 + 8 < max_row as u32 {
                *imp.row_offset.borrow_mut() = x + 8;
            }
            imp.tile_drawing.queue_draw();
        }));
    }

    fn setup_draw(
        &self,
        palette_data: Rc<RefCell<Palette>>,
        tile_data: Rc<RefCell<Tileset>>,
        bg_mode: Rc<RefCell<BGMode>>,
    ) {
        let imp = self.imp();
        imp.tile_drawing.set_draw_func(
            clone!(@weak palette_data, @weak tile_data, @weak imp.row_offset as row_offset, @weak imp.tile_size as tile_size => move |_, cr, w, _| {
                // default color
                cr.set_source_rgb(0.4, 0.4, 0.4);
                let _ = cr.paint();

                let tile_w = w as f64 / 16.0;

                // 16 8x8 tiles per row
                let row_offset = *row_offset.borrow();
                let tiles = &tile_data.borrow().tiles;
                for i in 0..256 {
                    let ti = (i + row_offset * 16) as usize;
                    if ti >= tiles.len() {
                        break;
                    }
                    let x_offset = (i % 16) as f64 * tile_w;
                    let y_offset = (i / 16) as f64 * tile_w;
                    let _ = cr.save();
                    cr.translate(x_offset, y_offset);
                    tiles[ti].draw(cr, palette_data.clone(), None, &bg_mode.borrow());
                    let _ = cr.restore();
                }

                // draw selected tile outline
                let _ = cr.save();
                cr.translate(0.0, -(row_offset as f64) * tile_w);
                let idx = tile_data.borrow().get_idx();
                match *tile_size.borrow() {
                    TileSize::Eight => {
                        let x = (idx % 16) as f64 * tile_w;
                        let y = (idx / 16) as f64 * tile_w;
                        cr.rectangle(x, y, tile_w, tile_w);
                        cr.set_source_rgb(0.8, 0.8, 0.0);
                    },
                    TileSize::Sixteen => {
                        let x = (idx % 16) as f64 * tile_w;
                        let y = (idx / 16) as f64 * tile_w;
                        cr.rectangle(x, y, tile_w * 2.0, tile_w * 2.0);
                        if idx % 16 == 15 {
                            // wrap around
                            cr.rectangle(-tile_w, y + tile_w, tile_w * 2.0, tile_w * 2.0);
                        }
                        if tile_data.borrow().is_valid_16() {
                            cr.set_source_rgb(0.8, 0.8, 0.0);
                        } else {
                            cr.set_source_rgb(0.5, 0.5, 0.5);
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
}
