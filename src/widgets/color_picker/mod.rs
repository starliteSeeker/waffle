mod imp;

use gtk::glib::{self, clone};
use gtk::prelude::*;
use gtk::subclass::prelude::*;

use crate::data::{color::Color, palette::Palette};
use crate::widgets::window::Window;

glib::wrapper! {
    pub struct ColorPicker(ObjectSubclass<imp::ColorPicker>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl ColorPicker {
    pub fn handle_action(&self, state: &Window) {
        fn modify_color(state: &Window, f: impl Fn(Color) -> Color) {
            let curr_color = state.picker_color().expect("picker_color not initialized");
            let curr_color = Color::from_variant(&curr_color).expect("picker_color is not Color");
            let new_color = f(curr_color);
            if curr_color != new_color {
                state.set_picker_color(new_color.to_variant());
            }
        }

        // rgb sliders
        let imp = self.imp();
        imp.red_slider.connect_change_value(
            clone!(@weak state => @default-return (false.into()), move |_, _, val| {
                // change red value of picker color
                modify_color(&state, |c| c.with_red(val.round() as u8));
                false.into()
            }),
        );
        imp.green_slider.connect_change_value(
            clone!(@weak state => @default-return (false.into()), move |_, _, val| {
                // change green value of picker color
                modify_color(&state, |c| c.with_green(val.round() as u8));
                false.into()
            }),
        );
        imp.blue_slider.connect_change_value(
            clone!(@weak state => @default-return (false.into()), move |_, _, val| {
                // change blue value of picker color
                modify_color(&state, |c| c.with_blue(val.round() as u8));
                false.into()
            }),
        );
    }

    pub fn render_widget(&self, state: &Window) {
        state.connect_picker_color_notify(clone!(@weak self as this => move |_| {
            this.imp().color_square.queue_draw();
        }));

        self.imp().color_square
            .set_draw_func(clone!(@weak state => move |_, cr, _, _| {
                let curr_color = state.picker_color().expect("picker_color not initialized");
                let curr_color = Color::from_variant(&curr_color).expect("picker_color is not Color");
                let (r, g, b) = curr_color.to_cairo();
                cr.set_source_rgb(r, g, b);
                let _ = cr.paint();
            }));
    }

    /*
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
    */
}
