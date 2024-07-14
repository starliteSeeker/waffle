use std::cell::{OnceCell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

use glib::clone;
use glib::subclass::{InitializingObject, Signal};
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{Button, DrawingArea, DropDown, Label, ScrolledWindow};

use crate::data::list_items::BGMode;

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/com/example/waffle/palette_picker.ui")]
#[properties(wrapper_type = super::PalettePicker)]
pub struct PalettePicker {
    #[template_child]
    pub palette_scroll: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub palette_drawing: TemplateChild<DrawingArea>,
    #[template_child]
    pub bpp_select: TemplateChild<DropDown>,

    #[template_child]
    pub color_idx_label: TemplateChild<Label>,

    #[property(get, set, nullable)]
    pub file: RefCell<Option<PathBuf>>,

    pub bg_mode: OnceCell<Rc<RefCell<BGMode>>>,

    #[template_child]
    pub test_button: TemplateChild<Button>,
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

        // redraw when bpp setting changed
        self.bpp_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                this.palette_drawing.queue_draw();
            }));

        self.test_button
            .connect_clicked(clone!(@weak self as this => move |_: &Button| {
                // this.palette_reload_btn.set_sensitive(false);
                this.obj().set_file(None::<PathBuf>);
                // println!("{}", this.palette_reload_btn.is_sensitive());
            }));
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("palette-changed").build(),
                // parameter: new-idx (index of color 0 of new palette)
                Signal::builder("palette-idx-changed")
                    .param_types([u8::static_type()])
                    .build(),
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
