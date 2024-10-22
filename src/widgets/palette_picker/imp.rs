use glib::subclass::InitializingObject;
use gtk::glib;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{DrawingArea, Label, ScrolledWindow};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/palette_picker.ui")]
pub struct PalettePicker {
    #[template_child]
    pub palette_scroll: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub palette_drawing: TemplateChild<DrawingArea>,
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

        // initialize label
        self.obj().set_label(0);
    }
}
impl WidgetImpl for PalettePicker {}
impl BoxImpl for PalettePicker {}
