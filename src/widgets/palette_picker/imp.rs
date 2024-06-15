use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use glib::clone;
use glib::subclass::{InitializingObject, Signal};
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::CompositeTemplate;
use gtk::{DrawingArea, DropDown, GestureClick, Label, ScrolledWindow};

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
    #[property(get, set)]
    pub color_idx: Rc<RefCell<u8>>,
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

        // bind color_idx_label
        self.obj()
            .bind_property("color_idx", &self.color_idx_label.get(), "label")
            .transform_to(|_, idx: u8| Some(format!("${:02X} / $FF", idx)))
            .sync_create()
            .build();

        // redraw when bpp setting changed
        self.bpp_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                this.palette_drawing.queue_draw();
            }));

        // redraw when color_idx changed
        self.obj()
            .connect_color_idx_notify(clone!(@weak self as this => move |_| {
                this.palette_drawing.queue_draw();
            }));

        // click event
        let gesture = GestureClick::new();
        gesture.connect_released(clone!(@weak self as this => move |_, _, x, y| {
            let yy = y + this.palette_scroll.vadjustment().value();
            let idx = (yy / 24.0) as u8 * 16 + (x / 24.0) as u8;
            this.color_idx.set(idx);
            this.obj().notify_color_idx();
        }));
        self.palette_scroll.add_controller(gesture);
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
impl WidgetImpl for PalettePicker {}
impl BoxImpl for PalettePicker {}
