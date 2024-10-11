use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

use glib::subclass::{InitializingObject, Signal};
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate, DrawingArea, DropDown, Label, StringList};

use strum::IntoEnumIterator;

use crate::data::list_items::TileSize;
use crate::data::tiles::Tileset;

#[derive(Properties, CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/tile_picker.ui")]
#[properties(wrapper_type = super::TilePicker)]
pub struct TilePicker {
    #[template_child]
    pub tile_drawing: TemplateChild<DrawingArea>,
    #[template_child]
    pub tile_prev: TemplateChild<Button>,
    #[template_child]
    pub tile_next: TemplateChild<Button>,
    #[template_child]
    pub tile_idx_label: TemplateChild<Label>,

    #[template_child]
    pub tile_size_select: TemplateChild<DropDown>,
    #[template_child]
    pub tile_size_items: TemplateChild<StringList>,

    #[property(get, set, nullable)]
    pub file: RefCell<Option<PathBuf>>,

    #[property(name = "row-offset", get, set)]
    row_offset_2: Cell<u32>,

    pub row_offset: Rc<RefCell<u32>>,
    pub tile_size: Rc<RefCell<TileSize>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for TilePicker {
    const NAME: &'static str = "TilePicker";
    type Type = super::TilePicker;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for TilePicker {
    fn constructed(&self) {
        self.parent_constructed();

        // initialize label
        self.obj()
            .set_index_label(0, Tileset::default().get_size() as u16 - 1);

        // populate StringList
        for i in TileSize::iter() {
            self.tile_size_items.append(&format!("{}", i));
        }
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("tile-idx-changed")
                    .param_types([u32::static_type()])
                    .build(),
                Signal::builder("tile-changed").build(),
                Signal::builder("tile-size-changed").build(),
                Signal::builder("bpp-changed")
                    .param_types([u8::static_type()])
                    .build(),
            ]
        })
    }
}
impl WidgetImpl for TilePicker {}
impl BoxImpl for TilePicker {}
