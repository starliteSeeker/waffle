mod imp;

use std::cell::RefCell;
use std::rc::Rc;

use gtk::glib::{self, closure_local, object::ObjectExt};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::data::palette::Palette;

glib::wrapper! {
    pub struct ColorPicker(ObjectSubclass<imp::ColorPicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl ColorPicker {
    pub fn setup_all<O: ObjectExt>(&self, palette_obj: O, palette_data: Rc<RefCell<Palette>>) {
        self.setup_signal_connection(palette_obj, palette_data);
    }

    fn setup_signal_connection<O: ObjectExt>(
        &self,
        palette_obj: O,
        palette_data: Rc<RefCell<Palette>>,
    ) {
        palette_obj.connect_closure(
            "color-idx-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: O, _: u8, mut red: u8, mut green: u8, mut blue: u8| {
                let Some(this) = this else {return};
                red = red.min(31);
                green = green.min(31);
                blue = blue.min(31);
                if red != this.red() || green != this.green() || blue != this.blue() {
                    this.set_red(red);
                    this.set_green(green);
                    this.set_blue(blue);
                    this.emit_by_name::<()>("color-changed", &[&red, &green, &blue]);
                }
            }),
        );
        palette_obj.connect_closure(
            "palette_changed",
            false,
            closure_local!(@weak-allow-none self as this, @weak-allow-none palette_data => move |_: O| {
                let Some(this) = this else {return};
                let Some(palette_data) = palette_data else {return};
                let (r, g, b) = palette_data.borrow().curr_color().to_tuple();
                if r != this.red() || g != this.green() || b != this.blue() {
                    this.set_red(r);
                    this.set_green(g);
                    this.set_blue(b);
                    this.emit_by_name::<()>("color-changed", &[&r, &g, &b]);
                }
            }),
        );

        // redraw self
        self.connect_closure(
            "color-changed",
            false,
            closure_local!(|this: Self, _: u8, _: u8, _: u8| {
                this.imp().color_square.queue_draw();
            }),
        );
    }
}
