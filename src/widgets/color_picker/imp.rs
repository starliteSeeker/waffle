use glib::subclass::InitializingObject;
use gtk::subclass::prelude::*;
use gtk::{glib, Adjustment, CompositeTemplate, DrawingArea, Scale};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/color_picker.ui")]
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
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for ColorPicker {
    const NAME: &'static str = "BGR555ColorPicker";
    type Type = super::ColorPicker;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for ColorPicker {}
impl WidgetImpl for ColorPicker {}
impl BoxImpl for ColorPicker {}
