mod imp;

use std::fs::File;
use std::path::PathBuf;

use strum::IntoEnumIterator;

use gio::{ActionEntry, SimpleActionGroup};
use glib::clone;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::GestureDrag;
use gtk::{gio, glib};

use crate::data::list_items::{BGModeTwo, DrawMode, TileSize, Zoom};
use crate::utils::*;
use crate::widgets::window::Window;
use crate::TILE_W;

glib::wrapper! {
    pub struct TilemapEditor(ObjectSubclass<imp::TilemapEditor>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TilemapEditor {
    pub fn handle_action(&self, state: &Window) {
        let imp = self.imp();

        // click/drag on editor
        self.setup_gesture(state);

        // tile setting toggle buttons
        imp.flip_x_btn
            .connect_active_notify(clone!(@weak imp => move |btn| {
                imp.curr_tile.borrow_mut().set_x_flip(btn.is_active());
            }));
        imp.flip_y_btn
            .connect_active_notify(clone!(@weak imp => move |btn| {
                imp.curr_tile.borrow_mut().set_y_flip(btn.is_active());
            }));
        imp.priority_btn
            .connect_active_notify(clone!(@weak imp => move |btn| {
                imp.curr_tile.borrow_mut().set_priority(btn.is_active());
            }));

        // change current tile
        state.connect_tileset_sel_idx_notify(clone!(@weak imp => move |state| {
            if let Err(e) = imp.curr_tile.borrow_mut().set_tile_idx_checked(state.tileset_sel_idx() as u16) {
                eprintln!("{e}");
            };
        }));

        // tilemap view settings
        imp.zoom_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                let imp = this.imp();
                let new_zoom = Zoom::iter().nth(imp.zoom_select.selected() as usize).expect("shouldn't happen");
                this.set_tilemap_zoom(new_zoom);
            }));
        imp.mode_select
            .connect_selected_notify(clone!(@weak state, @weak imp => move |_| {
                state.set_bg_mode(BGModeTwo::iter().nth(imp.mode_select.selected() as usize).expect("shouldn't happen"));
            }));
    }

    pub fn render_widget(&self, state: &Window) {
        self.connect_tilemap_zoom_notify(clone!(@weak self as this => move |_| {
            let imp = this.imp();
            let side_length = (TILE_W * 32.0 * this.tilemap_zoom().to_val()) as i32;
            imp.tilemap_drawing.set_content_width(side_length);
            imp.tilemap_drawing.set_content_height(side_length);

            imp.tilemap_drawing.queue_draw();
        }));

        state.connect_palette_data_notify(clone!(@weak self as this => move |_| {
            this.imp().tilemap_drawing.queue_draw();
        }));

        state.connect_tile_size_notify(clone!(@weak self as this => move |_| {
            this.imp().tilemap_drawing.queue_draw();
        }));

        state.connect_bg_mode_notify(clone!(@weak self as this => move |_| {
            this.imp().tilemap_drawing.queue_draw();
        }));

        state.connect_tilemap_data_notify(clone!(@weak self as this => move |_| {
            this.imp().tilemap_drawing.queue_draw();
        }));

        self.imp().tilemap_drawing.set_draw_func(
            clone!(@weak self as this, @weak state => move |_, cr, _, _| {
                let _ = cr.save();
                cr.set_antialias(gtk::cairo::Antialias::None);
                match this.tilemap_zoom() {
                    Zoom::Half => cr.scale(0.5, 0.5),
                    Zoom::One => (),
                    Zoom::Two => cr.scale(2.0, 2.0),
                }
                this.draw_tilemap(cr, &state);
                let _ = cr.restore();
            }),
        );
    }

    fn setup_gesture(&self, state: &Window) {
        let drag_event = GestureDrag::new();
        drag_event.connect_drag_begin(clone!(@weak self as this, @weak state => move |_, x, y| {
            let imp = this.imp();

            // calculate tile index
            let Some(new_idx) = this.cursor_to_idx(x, y) else {return};

            if imp.pen_draw_btn.is_active() {
                imp.curr_drag.replace(DrawMode::Pen);
                state.put_tile(new_idx as usize, &imp.curr_tile.borrow());
            } else if imp.rect_fill_btn.is_active() {
                let idx = (new_idx % 32, new_idx / 32);
                imp.curr_drag.replace(DrawMode::RectFill {
                    start: idx,
                    end: idx,
                });
            } else {
                eprintln!("draw mode not selected");
            }
        }));
        drag_event.connect_drag_update(
            clone!(@weak self as this, @weak state => move |drag, dx, dy| {
                let imp = this.imp();

                // calculate tile index
                let Some((x, y)) = drag.start_point() else {return};
                let Some(new_idx) = this.cursor_to_idx(x + dx, y + dy) else {return};

                let new_idx_2d = (new_idx % 32, new_idx / 32);

                match &mut *imp.curr_drag.borrow_mut() {
                    DrawMode::Pen => {
                        state.put_tile(new_idx, &imp.curr_tile.borrow());
                    },
                    DrawMode::RectFill { start: _, end } => {
                        if *end != new_idx_2d {
                            *end = new_idx_2d;
                            imp.tilemap_drawing.queue_draw();
                        }
                    }
                    _ => {
                        eprintln!("draw mode not selected");
                    }
                };
            }),
        );
        drag_event.connect_drag_end(clone!(@weak self as this, @weak state => move |_, _, _| {
            let imp = this.imp();
            match *imp.curr_drag.borrow() {
                DrawMode::RectFill {start, end} => {
                    let new_tile = this.imp().curr_tile.borrow();
                    state.modify_tilemap_data(move |tilemap| {
                        let ((x_min, x_max), (y_min, y_max)) = (
                            (start.0.min(end.0), start.0.max(end.0)),
                            (start.1.min(end.1), start.1.max(end.1)),
                        );
                        for i in y_min..=y_max {
                            for j in x_min..=x_max {
                                tilemap.0[i * 32 + j] = *new_tile;
                            }
                        }
                        return true;
                    });
                },
                _ => {},
            }

            imp.curr_drag.replace(DrawMode::None);
        }));
        self.imp().tilemap_drawing.add_controller(drag_event);
    }

    fn cursor_to_idx(&self, x: f64, y: f64) -> Option<usize> {
        let imp = self.imp();
        let tile_w = TILE_W * self.tilemap_zoom().to_val();

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

        let new_idx = tile_y.floor() as usize * 32 + tile_x.floor() as usize;
        Some(new_idx)
    }

    fn draw_tilemap(&self, cr: &gtk::cairo::Context, state: &Window) {
        let tileset = state.tileset_data();
        let curr_drag = self.imp().curr_drag.borrow();

        // fallback color
        cr.set_source_rgb(0.4, 0.4, 0.4);
        let _ = cr.paint();

        let curr_tile = self.imp().curr_tile.borrow();
        for (i, tile) in state.tilemap_data().0.iter().enumerate() {
            let ix = i % 32;
            let iy = i / 32;
            let x_offset = ix as f64 * TILE_W;
            let y_offset = iy as f64 * TILE_W;

            // decide which tile to draw
            let tile = if curr_drag.idx_in_range(ix, iy) {
                &curr_tile
            } else {
                tile
            };

            let _ = cr.save();
            cr.translate(x_offset, y_offset);
            if tile.x_flip() {
                cr.translate(TILE_W, 0.0);
                cr.scale(-1.0, 1.0);
            }
            if tile.y_flip() {
                cr.translate(0.0, TILE_W);
                cr.scale(1.0, -1.0);
            }

            let idx = tile.tile_idx().into();
            match state.tile_size() {
                TileSize::Eight => {
                    tileset.draw_tile(idx, cr, state, Some(tile.palette()));
                }
                TileSize::Sixteen => {
                    cr.scale(0.5, 0.5);
                    tileset.draw_tile(idx, cr, state, Some(tile.palette()));
                    cr.translate(TILE_W, 0.0);
                    tileset.draw_tile(idx + 1, cr, state, Some(tile.palette()));
                    cr.translate(-TILE_W, TILE_W);
                    tileset.draw_tile(idx + 16, cr, state, Some(tile.palette()));
                    cr.translate(TILE_W, 0.0);
                    tileset.draw_tile(idx + 17, cr, state, Some(tile.palette()));
                }
            }
            let _ = cr.restore();
        }
    }

    /*
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
    */
}
