use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::OnceLock;

use glib::subclass::{InitializingObject, Signal};
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{DrawingArea, Label, ScrolledWindow};

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/com/example/waffle/palette_picker.ui")]
#[properties(wrapper_type = super::PalettePicker)]
pub struct PalettePicker {
    #[template_child]
    pub palette_scroll: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub palette_drawing: TemplateChild<DrawingArea>,
    #[template_child]
    pub color_idx_label: TemplateChild<Label>,

    #[property(get, set, nullable)]
    pub file: RefCell<Option<PathBuf>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for PalettePicker {
    const NAME: &'static str = "PalettePicker";
    type Type = super::PalettePicker;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for PalettePicker {
    fn constructed(&self) {
        self.parent_constructed();

        // initialize label
        self.obj().set_label(0);
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("palette-changed").build(),
                // parameters: new-idx, red, green, blue
                Signal::builder("color-idx-changed")
                    .param_types([
                        u8::static_type(),
                        u8::static_type(),
                        u8::static_type(),
                        u8::static_type(),
                    ])
                    .build(),
            ]
        })
    }
}
impl WidgetImpl for PalettePicker {}
impl BoxImpl for PalettePicker {}
