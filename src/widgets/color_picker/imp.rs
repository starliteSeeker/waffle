use std::cell::Cell;
use std::sync::OnceLock;

use glib::clone;
use glib::subclass::InitializingObject;
use glib::subclass::Signal;
use glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Adjustment, CompositeTemplate, DrawingArea, Scale};

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/com/example/waffle/color_picker.ui")]
#[properties(wrapper_type = super::ColorPicker)]
pub struct ColorPicker {
    #[template_child]
    pub color_square: TemplateChild<DrawingArea>,
    #[template_child]
    pub red_slider: TemplateChild<Scale>,
    #[template_child]
    pub red_adj: TemplateChild<Adjustment>,
    #[template_child]
    pub green_slider: TemplateChild<Scale>,
    #[template_child]
    pub green_adj: TemplateChild<Adjustment>,
    #[template_child]
    pub blue_slider: TemplateChild<Scale>,
    #[template_child]
    pub blue_adj: TemplateChild<Adjustment>,

    #[property(get, set)]
    red: Cell<u8>,
    #[property(get, set)]
    green: Cell<u8>,
    #[property(get, set)]
    blue: Cell<u8>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ColorPicker {
    const NAME: &'static str = "BGR555ColorPicker";
    type Type = super::ColorPicker;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for ColorPicker {
    fn constructed(&self) {
        self.parent_constructed();

        self.setup_binds();
        self.setup_slider_change();

        // color square drawing
        self.color_square
            .get()
            .set_draw_func(clone!(@weak self as this => move |_, cr, _, _| {
                // convert 0~31 to 0.0~1.0
                let r = this.red.get() as f64 / 31.0;
                let g = this.green.get() as f64 / 31.0;
                let b = this.blue.get() as f64 / 31.0;
                cr.set_source_rgb(r, g, b);
                let _ = cr.paint();
            }));
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![Signal::builder("change-color")
                .param_types([u8::static_type(), u8::static_type(), u8::static_type()])
                .build()]
        })
    }
}
impl WidgetImpl for ColorPicker {}
impl BoxImpl for ColorPicker {}

impl ColorPicker {
    // bind rgb values with sliders
    fn setup_binds(&self) {
        let obj = self.obj();
        obj.bind_property("red", &self.red_adj.get(), "value")
            .bidirectional()
            .sync_create()
            .build();
        obj.bind_property("green", &self.green_adj.get(), "value")
            .bidirectional()
            .sync_create()
            .build();
        obj.bind_property("blue", &self.blue_adj.get(), "value")
            .bidirectional()
            .sync_create()
            .build();
    }

    // whenever slider value changes, redraw color square and emit signal "color-change"
    fn setup_slider_change(&self) {
        let obj = self.obj();
        self.red_slider
            .connect_change_value(clone!(@weak self as this, @weak obj =>
                @default-return (false.into()), move |_, _, val| {
                this.color_square.queue_draw();

                obj.emit_by_name::<()>("change-color", &[&(val.round() as u8), &this.green.get(), &this.blue.get()]);
                false.into()
            }));
        self.green_slider
            .connect_change_value(clone!(@weak self as this, @weak obj =>
                @default-return (false.into()), move |_, _, val| {
                this.color_square.queue_draw();

                obj.emit_by_name::<()>("change-color", &[&this.red.get(), &(val.round() as u8), &this.blue.get()]);
                false.into()
            }));
        self.blue_slider
            .connect_change_value(clone!(@weak self as this, @weak obj =>
                @default-return (false.into()), move |_, _, val| {
                this.color_square.queue_draw();

                obj.emit_by_name::<()>("change-color", &[&this.red.get(), &this.green.get(), &(val.round() as u8)]);
                false.into()
            }));
    }
}
