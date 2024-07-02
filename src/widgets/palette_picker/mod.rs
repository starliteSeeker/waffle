mod imp;

use std::cell::RefCell;
use std::rc::Rc;

use gio::{ActionEntry, SimpleActionGroup};
use glib::clone;
use glib::closure_local;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib};
use gtk::{FileChooserAction, FileChooserDialog, FileFilter, GestureClick, ResponseType};

use crate::data::color::Color;
use crate::data::palette::Palette;
use crate::widgets::window::Window;

const PAL_TILE_WIDTH: f64 = 24.0;

glib::wrapper! {
    pub struct PalettePicker(ObjectSubclass<imp::PalettePicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl PalettePicker {
    pub fn setup_all<O: ObjectExt>(
        &self,
        palette_data: Rc<RefCell<Palette>>,
        dialog_scope: Window,
        other: O,
    ) {
        self.setup_gesture(palette_data.clone());
        self.setup_actions(palette_data.clone(), dialog_scope);
        self.setup_signal_connection(other, palette_data.clone());
        self.setup_draw(palette_data.clone());
    }

    fn setup_gesture(&self, palette_data: Rc<RefCell<Palette>>) {
        // select palette
        let gesture = GestureClick::new();
        gesture.connect_released(clone!(@weak self as this => move |_, _, x, y| {
            // account for y scroll when calculating correct idx
            let yy = y + this.imp().palette_scroll.vadjustment().value();
            let new_idx = (yy / 24.0) as u8 * 16 + (x / 24.0) as u8;
            let old_idx = palette_data.borrow().sel_idx;
            // emit signals
            if new_idx != old_idx {
                palette_data.borrow_mut().sel_idx = new_idx;
                let (r, g, b) = palette_data.borrow().get_curr().to_tuple();
                this.emit_by_name::<()>("color-idx-changed", &[&new_idx, &r, &g, &b]);
            }
            if new_idx / 4 != old_idx / 4 {
                this.emit_by_name::<()>("palette-idx-changed", &[&(new_idx - new_idx % 4)]);
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
                                    let old_idx = palette_data.borrow().sel_idx;
                                    *palette_data.borrow_mut() = p;
                                    palette_data.borrow_mut().sel_idx = old_idx;
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

        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_open]);
        parent.insert_action_group("palette", Some(&actions));
    }

    fn setup_signal_connection<O: ObjectExt>(
        &self,
        color_obj: O,
        palette_data: Rc<RefCell<Palette>>,
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
                if palette_data.borrow_mut().set_curr(red, green, blue) {
                    this.emit_by_name::<()>("palette-changed", &[]);
                }
            }),
        );
    }

    fn setup_draw(&self, palette_data: Rc<RefCell<Palette>>) {
        let imp = self.imp();
        imp.palette_drawing.set_draw_func(
            clone!(@weak palette_data, @weak imp => move |_, cr, _, _| {
                // default color
                cr.set_source_rgb(1.0, 0.0, 1.0);
                let _ = cr.paint();

                // draw palette
                let pal = &palette_data.borrow().pal;
                let bpp = imp.bpp_select.selected();
                cr.set_line_width(1.0);
                for i in 0..16 {
                    for j in 0..16 {
                        let x_offset = j as f64 * PAL_TILE_WIDTH;
                        let y_offset = i as f64 * PAL_TILE_WIDTH;
                        cr.rectangle(x_offset, y_offset, PAL_TILE_WIDTH, PAL_TILE_WIDTH);
                        let (red, green, blue) = pal[i * 16 + j].to_cairo();
                        cr.set_source_rgb(red, green, blue);
                        let _ = cr.fill();

                        // add marker for transparent palette color
                        let is_transparent = match bpp {
                            0 => j % 4 == 0,
                            1 => j == 0,
                            _ => panic!("invalid bpp dropdown value"),
                        };
                        if is_transparent {
                            cr.arc(x_offset + 12.0, y_offset + 12.0, 3.0, 0.0 , 2.0 * std::f64::consts::PI);
                            cr.set_source_rgb(0.8, 0.8, 0.8);
                            let _ = cr.fill_preserve();
                            cr.set_source_rgb(0.0, 0.0, 0.0);
                            let _ = cr.stroke();
                        }

                    }
                }

                // draw current palette group outline
                let sel_idx = palette_data.borrow().sel_idx;
                let x_idx = sel_idx % 16;
                let y_idx = sel_idx / 16;
                match bpp {
                    0 => {
                        let x_offset = (x_idx - x_idx % 4) as f64 * PAL_TILE_WIDTH;
                        let y_offset = y_idx as f64 * PAL_TILE_WIDTH;
                        cr.rectangle(x_offset, y_offset, PAL_TILE_WIDTH * 4.0, PAL_TILE_WIDTH);
                    },
                    1 => {
                        let x_offset = 0 as f64 * PAL_TILE_WIDTH;
                        let y_offset = y_idx as f64 * PAL_TILE_WIDTH;
                        cr.rectangle(x_offset, y_offset, PAL_TILE_WIDTH * 16.0, PAL_TILE_WIDTH);
                    },
                    _ => panic!("invalid bpp dropdown value"),
                }
                cr.clip_preserve();
                cr.set_line_width(2.0);
                cr.set_source_rgb(0.8, 0.8, 0.0);
                let _ = cr.stroke();

                // draw current color outline
                let x_offset = (x_idx) as f64 * PAL_TILE_WIDTH;
                let y_offset = (y_idx) as f64 * PAL_TILE_WIDTH;
                cr.rectangle(x_offset, y_offset, PAL_TILE_WIDTH, PAL_TILE_WIDTH);
                cr.clip_preserve();
                cr.set_line_width(4.0);
                cr.set_source_rgb(1.0, 1.0, 0.0);
                let _ = cr.stroke();
            }),
        );
    }
}
