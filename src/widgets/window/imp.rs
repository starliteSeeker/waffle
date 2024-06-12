use std::sync::OnceLock;

use glib::subclass::InitializingObject;
use glib::subclass::Signal;
use glib::{clone, closure_local};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, Button, CompositeTemplate};

use crate::widgets::color_picker::ColorPicker;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/window.ui")]
pub struct Window {
    #[template_child]
    pub test_button: TemplateChild<Button>,
    #[template_child]
    pub color_picker: TemplateChild<ColorPicker>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "WaffleGtkAppWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        ColorPicker::ensure_type();

        klass.bind_template();
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();

        self.color_picker.connect_closure(
            "change-color",
            false,
            closure_local!(move |_: ColorPicker, red: u32, green: u32, blue: u32| {
                println!("{} {} {}", red, green, blue);
            }),
        );

        self.color_picker.setup_set_color(self.obj().clone());
        self.test_button
            .connect_clicked(clone!(@weak self as this => move |_| {
                let red = 1_u32;
                let green = 2_u32;
                let blue = 3_u32;
                this.obj().emit_by_name::<()>("set-color",
                    &[&red.to_value(), &green.to_value(), &blue.to_value()]);
            }));
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![Signal::builder("set-color")
                .param_types([u32::static_type(), u32::static_type(), u32::static_type()])
                .build()]
        })
    }
}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}
