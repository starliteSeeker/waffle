use std::cell::Cell;
use std::rc::Rc;

use glib::clone;
use glib::subclass::InitializingObject;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate, Label};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/tilemap_editor.ui")]
pub struct TilemapEditor {
    #[template_child]
    pub map_btn: TemplateChild<Button>,
    #[template_child]
    pub map_lbl: TemplateChild<Label>,

    pub map_data: Rc<Cell<u8>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for TilemapEditor {
    const NAME: &'static str = "TilemapEditor";
    type Type = super::TilemapEditor;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for TilemapEditor {
    fn constructed(&self) {
        self.parent_constructed();
        self.map_btn
            .connect_clicked(clone!(@weak self as this => move |_| {
                this.map_lbl.set_label(&this.map_data.get().to_string());
            }));
    }
}
impl WidgetImpl for TilemapEditor {}
impl BoxImpl for TilemapEditor {}
