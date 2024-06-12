mod imp;

use gtk::glib::{self, closure_local, object::ObjectExt};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

glib::wrapper! {
    pub struct ColorPicker(ObjectSubclass<imp::ColorPicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl ColorPicker {
    pub fn setup_set_color<O: ObjectExt>(&self, other: O) {
        other.connect_closure(
            "set-color",
            false,
            closure_local!(@weak-allow-none self as this => move |_: O, mut red: u32, mut green: u32, mut blue: u32| {
                let Some(this) = this else {return};
                red = red.min(31);
                green = green.min(31);
                blue = blue.min(31);
                this.set_red(red);
                this.set_green(green);
                this.set_blue(blue);
                this.imp().color_square.queue_draw();
                println!("set to {red} {green} {blue}");
            }),
        );
    }
}
