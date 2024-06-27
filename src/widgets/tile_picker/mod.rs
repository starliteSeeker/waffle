mod imp;

use std::cell::RefCell;
use std::rc::Rc;

use gio::{ActionEntry, SimpleActionGroup};
use glib::{clone, closure_local};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{FileChooserAction, FileChooserDialog, FileFilter, GestureClick, ResponseType, Window};

use crate::data::list_items::TileSize;
use crate::data::palette::Palette;
use crate::data::tiles::Tileset;

glib::wrapper! {
    pub struct TilePicker(ObjectSubclass<imp::TilePicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TilePicker {
    pub fn setup_all<S: WidgetExt + IsA<Window>, P: WidgetExt>(
        &self,
        dialog_scope: Option<S>,
        pal: Rc<RefCell<Palette>>,
        tile_data: Rc<RefCell<Tileset>>,
        palette_obj: P,
    ) {
        self.setup_gesture(tile_data.clone());
        self.setup_actions(dialog_scope, tile_data.clone());
        self.setup_signal_connection(tile_data.clone(), palette_obj.clone());
        self.setup_draw(pal, tile_data);
    }

    fn setup_gesture(&self, tile_data: Rc<RefCell<Tileset>>) {
        let gesture = GestureClick::new();
        gesture.connect_released(clone!(@weak self as this => move |_, _, x, y| {
            // account for row offset when calculating correct idx
            let new_idx = (*this.imp().row_offset.borrow() + y as u32 / 24) * 16 + (x / 24.0) as u32;
            // emit signal
            if tile_data.borrow_mut().set_idx(new_idx) {
                this.emit_by_name::<()>("tile-idx-changed", &[]);
            }
        }));
        self.imp().tile_drawing.add_controller(gesture);
    }

    fn setup_actions<S: WidgetExt + IsA<Window>>(
        &self,
        scope: Option<S>,
        tile_data: Rc<RefCell<Tileset>>,
    ) {
        let scope_clone = scope.clone();
        let action_open = ActionEntry::builder("open")
            .activate(
                clone!(@weak self as this, @weak tile_data => move |_, _, _| {
                    let dialog = FileChooserDialog::new(
                        Some("Open Palette File"),
                        scope_clone.as_ref(),
                        FileChooserAction::Open,
                        &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
                    );

                    let bin_filter = FileFilter::new();
                    bin_filter.set_name(Some("Binary Files (.bin)"));
                    bin_filter.add_suffix("bin");
                    let all_filter = FileFilter::new();
                    all_filter.set_name(Some("All Files"));
                    all_filter.add_pattern("*");
                    dialog.add_filter(&bin_filter);
                    dialog.add_filter(&all_filter);

                    dialog.connect_response(move |d: &FileChooserDialog, response: ResponseType| {
                        if response == ResponseType::Ok {
                            // second dialog prompting for bit depth
                            // TODO: pretend it's 2bpp
                            /*
                            let dialog = Dialog::with_buttons(
                                Some("Select Bits per Pixel"),
                                Some(d),
                                DialogFlags::DESTROY_WITH_PARENT,
                                &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
                                dialog.connect_response(move |d: &FileChooserDialog, response: ResponseType| {
                            );
                            */
                            let file = d.file().expect("Couldn't get file");
                            let filename = file.path().expect("Couldn't get file path");
                            match Tileset::from_path(&filename) {
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                }
                                Ok(t) => {
                                    println!("load tileset: {filename:?}");
                                    *tile_data.borrow_mut() = t;
                                    this.emit_by_name::<()>("tile-changed", &[]);
                                }
                            }
                        }
                        d.close();
                    });
                    dialog.show();
                }),
            )
            .build();
        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_open]);
        match scope {
            Some(s) => s.insert_action_group("tiles", Some(&actions)),
            None => self.insert_action_group("tiles", Some(&actions)),
        }
    }

    fn setup_signal_connection<P: WidgetExt>(
        &self,
        tile_data: Rc<RefCell<Tileset>>,
        palette_obj: P,
    ) {
        // redraw self
        self.connect_closure(
            "tile-idx-changed",
            false,
            closure_local!(|this: Self| {
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
            closure_local!(@weak-allow-none self as this => move |_: P| {
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

        // update tile index label
        self.connect_closure(
            "tile-idx-changed",
            false,
            closure_local!(@weak-allow-none tile_data => move |this: Self| {
                let Some(tile_data) = tile_data else {return};
                this.imp().tile_idx_label.set_label(&format!("${:03X} / ${:03X}", tile_data.borrow().get_idx(), tile_data.borrow().get_size()));
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

    fn setup_draw(&self, palette_data: Rc<RefCell<Palette>>, tile_data: Rc<RefCell<Tileset>>) {
        let imp = self.imp();
        imp.tile_drawing.set_draw_func(
            clone!(@weak palette_data, @weak tile_data, @weak imp.row_offset as row_offset, @weak imp.tile_size as tile_size => move |_, cr, w, _| {
                // default color
                cr.set_source_rgb(0.4, 0.4, 0.4);
                let _ = cr.paint();

                let tile_w = w as f64 / 16.0;
                let pxl_w = tile_w / 8.0;

                // 16 8x8 tiles per row
                // TODO: assume 2bpp for now
                // collect pixels with same color, then draw the pixels together
                let mut rects = vec![Vec::new(); 4];
                let row_offset = *row_offset.borrow();
                let tiles = &tile_data.borrow().tiles;
                for i in 0..256 {
                    let ti = (i + row_offset * 16) as usize;
                    if ti >= tiles.len() {
                        break;
                    }
                    // top left corner of tile
                    let x_off = (i % 16) as f64 * tile_w;
                    let y_off = (i / 16) as f64 * tile_w;
                    let chr = tiles[ti].chr;
                    for (j, c) in chr.into_iter().enumerate() {
                        // top left corner of pixel
                        let xx_off = (j % 8) as f64 * pxl_w;
                        let yy_off = (j / 8) as f64 * pxl_w;
                        rects[c as usize].push((x_off + xx_off, y_off + yy_off));
                    }
                }

                for (i, v) in rects.into_iter().enumerate() {
                    for (x, y) in v {
                        cr.rectangle(x, y, pxl_w, pxl_w);
                    }
                    let (r, g, b) = palette_data.borrow().get_relative(i as u8).to_cairo();
                    cr.set_source_rgb(r, g, b);
                    let _ = cr.fill();
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
