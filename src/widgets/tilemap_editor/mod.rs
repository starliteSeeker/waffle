mod imp;

use std::cell::RefCell;
use std::fs::File;
use std::path::PathBuf;
use std::rc::Rc;

use strum::IntoEnumIterator;

use gio::{ActionEntry, SimpleActionGroup};
use glib::clone;
use glib::closure_local;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::GestureDrag;
use gtk::{gio, glib};

use crate::data::list_items::{BGMode, BGModeTwo, Bpp, TileSize, Zoom};
use crate::data::palette::Palette;
use crate::data::tilemap::Tilemap;
use crate::data::tiles::Tileset;
use crate::utils::*;
use crate::widgets::window::Window;

glib::wrapper! {
    pub struct TilemapEditor(ObjectSubclass<imp::TilemapEditor>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TilemapEditor {
    pub fn setup_all<P: WidgetExt, T: WidgetExt>(
        &self,
        palette_data: Rc<RefCell<Palette>>,
        tile_data: Rc<RefCell<Tileset>>,
        tile_size: Rc<RefCell<TileSize>>,
        palette_obj: P,
        tile_obj: T,
        parent: Window,
    ) {
        self.setup_gesture();
        self.setup_draw(palette_data, tile_data.clone(), tile_size);
        self.setup_signal_connection(palette_obj, tile_obj);
        self.setup_actions(parent);
    }

    fn setup_gesture(&self) {
        let drag_event = GestureDrag::new();
        drag_event.connect_drag_begin(clone!(@weak self as this => move |_, x, y| {
            let imp = this.imp();

            // calculate tile index
            let Some(new_idx) = this.cursor_to_idx(x, y) else {return};

            if imp.rect_fill_btn.is_active() {
                let (idx_x, idx_y) = (new_idx % 32, new_idx / 32);
                *imp.curr_drag.borrow_mut() = Some(((idx_x, idx_y), (idx_x, idx_y)));
                this.emit_by_name::<()>("tilemap-changed", &[]);
            } else if imp.map_data.borrow_mut().set_tile(new_idx, &*imp.curr_tile.borrow()) {
                this.emit_by_name::<()>("tilemap-changed", &[]);
            }
        }));
        drag_event.connect_drag_update(clone!(@weak self as this => move |drag, dx, dy| {
            let imp = this.imp();

            // calculate tile index
            let Some((x, y)) = drag.start_point() else {return};
            let Some(new_idx) = this.cursor_to_idx(x + dx, y + dy) else {return};

            let (idx_x, idx_y) = (new_idx % 32, new_idx / 32);

            let mut curr_drag = imp.curr_drag.borrow_mut();

            // emit signal
            if imp.rect_fill_btn.is_active() {
                if curr_drag.is_some_and(|(_, end)| end != (idx_x, idx_y)) {
                    *curr_drag = curr_drag.map(|(start, _)| (start, (idx_x, idx_y)));
                    this.emit_by_name::<()>("tilemap-changed", &[]);
                }
            }
            else if imp.map_data.borrow_mut().set_tile(new_idx, &*imp.curr_tile.borrow())
            {
                this.emit_by_name::<()>("tilemap-changed", &[]);
            }
        }));
        drag_event.connect_drag_end(clone!(@weak self as this => move |_, _, _| {
            let imp = this.imp();
            if imp.rect_fill_btn.is_active() {
                // sort rectangle fill boundaries
                let ((x_min, x_max), (y_min, y_max)) = if let Some(((x1, y1), (x2, y2))) = *imp.curr_drag.borrow() {
                    (
                        (x1.min(x2) as usize, x1.max(x2) as usize),
                        (y1.min(y2) as usize, y1.max(y2) as usize),
                    )
                } else {
                    return;
                };
                // update tilemap
                let mut map_data = imp.map_data.borrow_mut();
                for i in y_min..=y_max {
                    for j in x_min..=x_max {
                        map_data.set_tile((i * 32 + j) as u32, &*imp.curr_tile.borrow());
                    }
                }
                // reset selection
                *imp.curr_drag.borrow_mut() = None;
                this.emit_by_name::<()>("tilemap-changed", &[]);
            }
        }));
        self.imp().tilemap_drawing.add_controller(drag_event);
    }

    fn cursor_to_idx(&self, x: f64, y: f64) -> Option<u32> {
        let imp = self.imp();
        let tile_w = crate::TILE_W * imp.zoom_level.borrow().to_val();

        let scroll = &imp.tilemap_scroll;
        if x < scroll.hadjustment().value()
            || y < scroll.vadjustment().value()
            || x >= scroll.width() as f64 + scroll.hadjustment().value()
            || y >= scroll.height() as f64 + scroll.vadjustment().value()
        {
            // cursor position outside of tilemap_scroll
            return None;
        }

        let (tile_x, tile_y) = (x / tile_w, y / tile_w);
        if tile_x < 0.0 || tile_y < 0.0 || tile_x >= 32.0 || tile_y >= 32.0 {
            // cursor position outside of tilemap drawing
            return None;
        }

        let new_idx = tile_y.floor() as u32 * 32 + tile_x.floor() as u32;
        Some(new_idx)
    }

    fn setup_draw(
        &self,
        palette_data: Rc<RefCell<Palette>>,
        tile_data: Rc<RefCell<Tileset>>,
        tile_size: Rc<RefCell<TileSize>>,
    ) {
        let imp = self.imp();
        imp.tilemap_drawing.set_draw_func(
            clone!(@weak imp, @weak palette_data, @weak tile_data, @weak tile_size => move |_, cr, _, _| {
                let _ = cr.save();
                cr.set_antialias(gtk::cairo::Antialias::None);
                match *imp.zoom_level.borrow() {
                    Zoom::Half => cr.scale(0.5, 0.5),
                    Zoom::One => (),
                    Zoom::Two => cr.scale(2.0, 2.0),
                }
                imp.map_data.borrow().draw(cr, palette_data, tile_data, imp.bg_mode.clone(), tile_size, *imp.curr_drag.borrow(), *imp.curr_tile.borrow());
                let _ = cr.restore();
            }),
        );
    }

    fn setup_signal_connection<P: WidgetExt, T: WidgetExt>(&self, palette_obj: P, tile_obj: T) {
        palette_obj.connect_closure(
            "palette-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: P| {
                let Some(this) = this else {return};
                this.imp().tilemap_drawing.queue_draw();
            }),
        );
        palette_obj.connect_closure(
            "color-idx-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: P, new_idx: u8, _: u8, _: u8, _: u8| {
                let Some(this) = this else {return};
                let bg_mode = this.imp().bg_mode.borrow();
                let palette_idx = (new_idx / bg_mode.bpp().to_val()) % 8;
                this.imp().curr_tile.borrow_mut().set_palette(palette_idx.min(7));
            }),
        );

        tile_obj.connect_closure(
            "tile-idx-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: T, new_idx: u32| {
                let Some(this) = this else {return};
                this.imp().curr_tile.borrow_mut().set_tile_idx(new_idx as u16);
            }),
        );

        tile_obj.connect_closure(
            "tile-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: T| {
                let Some(this) = this else {return};
                this.imp().tilemap_drawing.queue_draw();
            }),
        );
        tile_obj.connect_closure(
            "tile-size-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: T| {
                let Some(this) = this else {return};
                this.imp().tilemap_drawing.queue_draw();
            }),
        );
        tile_obj.connect_closure(
            "bpp-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: T, bpp: u8| {
                let Some(this) = this else {return};
                let bpp = Bpp::iter().nth(bpp as usize).expect("shouldn't happen");
                match bpp {
                    Bpp::Two => {
                        this.imp().mode_select.set_sensitive(true);
                        let bpp2 = BGModeTwo::iter().nth(this.imp().mode_select.selected() as usize).expect("shouldn't happen");
                        *this.imp().bg_mode.borrow_mut() = BGMode::Two(bpp2);
                        this.emit_by_name::<()>("bg-mode-changed", &[]);
                    },
                    Bpp::Four => {
                        // disable dropdown
                        this.imp().mode_select.set_sensitive(false);
                        *this.imp().bg_mode.borrow_mut() = BGMode::Four;
                        this.emit_by_name::<()>("bg-mode-changed", &[]);
                    },
                }
            }),
        );
    }

    fn setup_actions(&self, parent: Window) {
        let action_open = ActionEntry::builder("open")
            .activate(clone!(@weak self as this, @weak parent => move |_, _, _| {
                file_open_dialog(parent, move |path| {
                    match Tilemap::from_path(&path) {
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                        Ok(t) => {
                            println!("load tilemap: {path:?}");
                            *this.imp().map_data.borrow_mut() = t;
                            this.set_file(Some(path));
                            this.emit_by_name::<()>("tilemap-changed", &[]);
                        }
                    }
                });
            }))
            .build();

        let action_save = ActionEntry::builder("save")
            .activate(clone!(@weak self as this, @weak parent => move |_, _, _| {
                let Some(filepath) = this.file() else {return};
                println!("save tilemap: {filepath:?}");
                match File::create(filepath) {
                    Ok(f) => {
                        let _ = this.imp().map_data.borrow().write_to_file(&f);
                    },
                    Err(e) => eprintln!("Error saving file: {e}"),
                }
            }))
            .build();

        let action_save_as = ActionEntry::builder("saveas")
            .activate(clone!(@weak self as this, @weak parent => move |_, _, _| {
                file_save_dialog(parent, move |_, filepath| {
                    println!("save tilemap: {filepath:?}");
                    match File::create(filepath.clone()) {
                        Ok(f) => {
                            let _ = this.imp().map_data.borrow().write_to_file(&f);
                            this.set_file(Some(filepath));
                        },
                        Err(e) => eprintln!("Error saving file: {e}"),
                    }
                });
            }))
            .build();

        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_open, action_save, action_save_as]);

        // bind file to action
        let save = actions.lookup_action("save").unwrap();
        self.bind_property("file", &save, "enabled")
            .transform_to(|_, file: Option<PathBuf>| Some(file.is_some()))
            .sync_create()
            .build();

        parent.insert_action_group("tilemap", Some(&actions));
    }
}
