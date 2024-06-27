use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use glib::clone;
use glib::subclass::{InitializingObject, Signal};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{DrawingArea, DropDown, Label, ScrolledWindow};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/palette_picker.ui")]
pub struct PalettePicker {
    #[template_child]
    pub palette_scroll: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub palette_drawing: TemplateChild<DrawingArea>,
    #[template_child]
    pub bpp_select: TemplateChild<DropDown>,

    #[template_child]
    pub color_idx_label: TemplateChild<Label>,
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

impl ObjectImpl for PalettePicker {
    fn constructed(&self) {
        self.parent_constructed();

        // redraw when bpp setting changed
        self.bpp_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                this.palette_drawing.queue_draw();
            }));
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("palette-changed").build(),
                Signal::builder("palette-idx-changed").build(),
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
