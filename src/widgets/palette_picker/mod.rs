mod imp;

use std::cell::RefCell;
use std::rc::Rc;

use glib::clone;
use glib::closure_local;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::data::color::Color;
use crate::data::palette::Palette;

const PAL_TILE_WIDTH: f64 = 24.0;

glib::wrapper! {
    pub struct PalettePicker(ObjectSubclass<imp::PalettePicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl PalettePicker {
    pub fn setup_emit_set_color(&self, palette_data: Rc<RefCell<Palette>>) {
        self.connect_color_idx_notify(clone!(@weak palette_data => move |picker| {
            // emit set-color along with selected color
            let idx = picker.color_idx() as usize;
            let (red, green, blue) = palette_data.borrow_mut().pal[idx].to_tuple();
            picker.emit_by_name::<()>("set-color", &[&red, &green, &blue]);
        }));
    }

    pub fn setup_change_color<O: ObjectExt>(&self, other: O, palette_data: Rc<RefCell<Palette>>) {
        other.connect_closure(
            "change-color",
            false,
            closure_local!(@weak-allow-none self as this, @weak-allow-none palette_data => move |_: O, red: u32, green: u32, blue: u32| {
                // update color at color_idx
                let Some(this) = this else {return};
                let Some(palette_data) = palette_data else {return};
                let idx = this.color_idx() as usize;
                palette_data.borrow_mut().pal[idx] = Color::new(red, green, blue);
                this.imp().palette_drawing.queue_draw();
            }),
        );
    }

    pub fn setup_palette_data(&self, palette_data: Rc<RefCell<Palette>>) {
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
                        let x_offset =j as f64 * PAL_TILE_WIDTH;
                        let y_offset =i as f64 * PAL_TILE_WIDTH;
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

                // draw current palette group box
                let x_idx = *imp.color_idx.borrow() % 16;
                let y_idx = *imp.color_idx.borrow() / 16;
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

                // draw current color box
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
