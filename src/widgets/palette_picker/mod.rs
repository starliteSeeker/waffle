mod imp;

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

use std::fs::File;

use gio::{ActionEntry, SimpleActionGroup};
use glib::clone;
use glib::closure_local;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{FileChooserAction, FileChooserDialog, FileFilter, GestureClick, ResponseType};

use crate::data::list_items::{BGMode, Bpp};
use crate::data::palette::Palette;
use crate::widgets::window::Window;
use crate::TILE_W;

glib::wrapper! {
    pub struct PalettePicker(ObjectSubclass<imp::PalettePicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl PalettePicker {
    pub fn setup_all<O: ObjectExt, M: WidgetExt>(
        &self,
        palette_data: Rc<RefCell<Palette>>,
        dialog_scope: Window,
        other: O,
        bg_mode: Rc<RefCell<BGMode>>,
        map_obj: M,
    ) {
        let _ = self.imp().bg_mode.set(bg_mode.clone());

        self.setup_gesture(palette_data.clone());
        self.setup_actions(palette_data.clone(), dialog_scope);
        self.setup_signal_connection(other, palette_data.clone(), map_obj);
        self.setup_draw(palette_data.clone());
    }

    fn setup_gesture(&self, palette_data: Rc<RefCell<Palette>>) {
        // select palette
        let gesture = GestureClick::new();
        gesture.connect_released(clone!(@weak self as this => move |_, _, x, y| {
            // account for y scroll when calculating correct idx
            let yy = y + this.imp().palette_scroll.vadjustment().value();
            let new_idx = (yy / TILE_W) as u8 * 16 + (x / TILE_W) as u8;
            let bg_mode = &*this.imp().bg_mode.get().unwrap().borrow();
            let (pal_changed, col_changed) = palette_data.borrow_mut().set_idx(new_idx, bg_mode);

            // emit signals
            if col_changed {
                let (r, g, b) = palette_data.borrow().curr_color(bg_mode).to_tuple();
                this.emit_by_name::<()>("color-idx-changed", &[&new_idx, &r, &g, &b]);
            }
            // TODO bpp stuff
            if pal_changed {
                this.emit_by_name::<()>("palette-idx-changed", &[&(palette_data.borrow().pal_sel)]);
            }
        }));
        self.imp().palette_scroll.add_controller(gesture);
    }

    fn setup_actions(&self, palette_data: Rc<RefCell<Palette>>, parent: Window) {
        // open file
        let action_open = ActionEntry::builder("open")
            .activate(
                clone!(@weak self as this, @weak palette_data, @weak parent => move |_, _, _| {
                    let dialog = FileChooserDialog::new(
                        Some("Open Palette File"),
                        Some(&parent),
                        FileChooserAction::Open,
                        &[("Open", ResponseType::Ok), ("Cancel", ResponseType::Cancel)],
                    );

                    // *.bin file filter and all file filter
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
                            // load file
                            let file = d.file().expect("Couldn't get file");
                            let filename = file.path().expect("Couldn't get file path");
                            match Palette::from_path(&filename) {
                                Err(e) => {
                                    eprintln!("Error: {}", e);
                                }
                                Ok(p) => {
                                    println!("load palette: {filename:?}");
                                    palette_data.borrow_mut().pal = p.pal;
                                    this.set_file(Some(filename));
                                    this.emit_by_name::<()>("palette-changed", &[]);
                                }
                            };
                        }

                        d.close();
                    });

                    dialog.show();
                }),
            )
            .build();

        let action_reload = ActionEntry::builder("reload")
            .activate(
                clone!(@weak self as this, @weak palette_data => move |_, _, _| {
                    let Some(file) = this.file() else {
                        eprintln!("No palette file currently open");
                        return;
                    };

                    match Palette::from_path(&file) {
                        Err(e) => {
                            eprintln!("Error: {}", e);
                        }
                        Ok(p) => {
                            println!("load palette: {file:?}");
                            palette_data.borrow_mut().pal = p.pal;
                            this.emit_by_name::<()>("palette-changed", &[]);
                        }
                    };
                }),
            )
            .build();

        let action_save = ActionEntry::builder("save")
            .activate(
                clone!(@weak self as this, @weak palette_data => move |_, _, _| {
                    let Some(filepath) = this.file() else {return};
                    println!("save palette: {filepath:?}");
                    match File::create(filepath) {
                        Ok(mut f) => {
                            for c in palette_data.borrow().pal {
                                let _ = f.write_all(&c.into_bytes());
                            }
                        },
                        Err(e) => eprintln!("Error saving file: {e}"),
                    }
                }),
            )
            .build();

        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_open, action_reload, action_save]);
        parent.insert_action_group("palette", Some(&actions));
    }

    fn setup_signal_connection<O: ObjectExt, M: WidgetExt>(
        &self,
        color_obj: O,
        palette_data: Rc<RefCell<Palette>>,
        map_obj: M,
    ) {
        // update color_idx_label
        let imp = self.imp();
        let color_idx_label = imp.color_idx_label.get();
        self.connect_closure(
            "color-idx-changed",
            false,
            closure_local!(@weak-allow-none color_idx_label => move |_: Self, new_idx: u8, _: u8, _: u8, _: u8| {
                let Some(color_idx_label) = color_idx_label else {return};
                color_idx_label.set_label(&format!("${:02X} / $FF", new_idx));
            }),
        );

        // redraw self
        self.connect_closure(
            "palette-changed",
            false,
            closure_local!(move |this: Self| {
                this.imp().palette_drawing.queue_draw();
            }),
        );
        self.connect_closure(
            "color-idx-changed",
            false,
            closure_local!(move |this: Self, _: u8, _: u8, _: u8, _: u8| {
                this.imp().palette_drawing.queue_draw();
            }),
        );

        // update palette_drawing
        color_obj.connect_closure(
            "color-changed",
            false,
            closure_local!(@weak-allow-none self as this, @weak-allow-none palette_data => move |_: O, red: u8, green: u8, blue: u8| {
                // update color at color_idx
                let Some(this) = this else {return};
                let Some(palette_data) = palette_data else {return};
                let Some(bg_mode) = this.imp().bg_mode.get() else {return};

                if palette_data.borrow_mut().set_curr(red, green, blue, &bg_mode.borrow()) {
                    this.emit_by_name::<()>("palette-changed", &[]);
                }
            }),
        );

        map_obj.connect_closure(
            "bg-mode-changed",
            false,
            closure_local!(@weak-allow-none self as this, @weak-allow-none palette_data => move |_: M| {
                let Some(this) = this else {return};
                let Some(palette_data) = palette_data else {return};
                let Some(bg_mode) = this.imp().bg_mode.get() else {return};
                this.imp().palette_drawing.queue_draw();

                let (r, g, b) = palette_data.borrow().curr_color(&bg_mode.borrow()).to_tuple();
                let curr_idx = palette_data.borrow().curr_idx(&bg_mode.borrow());
                this.emit_by_name::<()>("color-idx-changed", &[&curr_idx, &r, &g, &b]);
            }),
        );
    }

    fn setup_draw(&self, palette_data: Rc<RefCell<Palette>>) {
        let imp = self.imp();
        imp.palette_drawing.set_draw_func(
            clone!(@weak palette_data, @weak imp => move |_, cr, x, y| {
                // default color
                cr.set_source_rgb(1.0, 0.0, 1.0);
                let _ = cr.paint();

                // draw palette
                let pal = &palette_data.borrow().pal;
                let bg_mode = imp.bg_mode.get().unwrap().borrow();
                cr.set_line_width(1.0);
                for i in 0..16 {
                    for j in 0..16 {
                        let x_offset = j as f64 * TILE_W;
                        let y_offset = i as f64 * TILE_W;
                        cr.rectangle(x_offset, y_offset, TILE_W, TILE_W);
                        let (red, green, blue) = pal[i * 16 + j].to_cairo();
                        cr.set_source_rgb(red, green, blue);
                        let _ = cr.fill();

                        // add marker for transparent palette color
                        let is_transparent = match bg_mode.bpp() {
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
                let y_offset = (bg_mode.palette_offset() / 16) as f64 * TILE_W;
                let width = 16.0 * TILE_W;
                let height = (bg_mode.bpp().to_val() * 8 / 16) as f64 * TILE_W;
                cr.rectangle(x_offset, y_offset, width, height);
                cr.rectangle(0.0, 0.0, x as f64, y as f64);
                let _ = cr.save();
                cr.set_fill_rule(gtk::cairo::FillRule::EvenOdd);
                cr.set_source_rgba(0.0, 0.0, 0.0, 0.7);
                let _ = cr.fill();
                let _ = cr.restore();

                // draw current palette group outline
                let pal_start_idx = bg_mode.palette_offset() + bg_mode.bpp().to_val() * palette_data.borrow().pal_sel;
                let x_offset = (pal_start_idx % 16) as f64 * TILE_W;
                let y_offset = (pal_start_idx / 16) as f64 * TILE_W;
                cr.rectangle(x_offset, y_offset, TILE_W * bg_mode.bpp().to_val() as f64, TILE_W);

                cr.clip_preserve();
                cr.set_line_width(2.0);
                cr.set_source_rgb(0.8, 0.8, 0.0);
                let _ = cr.stroke();

                // draw current color outline
                let x_offset = (palette_data.borrow().curr_idx(&bg_mode) % 16) as f64 * TILE_W;
                let y_offset = (palette_data.borrow().curr_idx(&bg_mode) / 16) as f64 * TILE_W;
                cr.rectangle(x_offset, y_offset, TILE_W, TILE_W);
                cr.clip_preserve();
                cr.set_line_width(4.0);
                cr.set_source_rgb(1.0, 1.0, 0.0);
                let _ = cr.stroke();
            }),
        );
    }
}
