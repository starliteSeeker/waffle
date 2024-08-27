use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use glib::subclass::InitializingObject;
use glib::subclass::Signal;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{glib, CompositeTemplate};

use crate::data::palette::Palette;
use crate::data::tiles::Tileset;
use crate::widgets::color_picker::ColorPicker;
use crate::widgets::palette_picker::PalettePicker;
use crate::widgets::tile_picker::TilePicker;
use crate::widgets::tilemap_editor::TilemapEditor;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/window.ui")]
pub struct Window {
    #[template_child]
    pub color_picker: TemplateChild<ColorPicker>,
    #[template_child]
    pub palette_picker: TemplateChild<PalettePicker>,
    #[template_child]
    pub tilemap_editor: TemplateChild<TilemapEditor>,
    #[template_child]
    pub tile_picker: TemplateChild<TilePicker>,

    pub palette_data: Rc<RefCell<Palette>>,
    pub tile_data: Rc<RefCell<Tileset>>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for Window {
    const NAME: &'static str = "WaffleGtkAppWindow";
    type Type = super::Window;
    type ParentType = gtk::ApplicationWindow;

    fn class_init(klass: &mut Self::Class) {
        ColorPicker::ensure_type();
        PalettePicker::ensure_type();
        TilemapEditor::ensure_type();

        klass.bind_template();
        klass.install_action("win.close", None, |window, _, _| window.close());
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

// Trait shared by all GObjects
impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();

        let bg_mode = self.tilemap_editor.imp().bg_mode.clone();

        self.palette_picker.setup_all(
            self.palette_data.clone(),
            self.obj().clone(),
            self.color_picker.clone(),
            bg_mode.clone(),
            self.tilemap_editor.clone(),
        );

        self.color_picker
            .setup_all(self.palette_picker.clone(), self.palette_data.clone());

        self.tile_picker.setup_all(
            self.obj().clone(),
            self.palette_data.clone(),
            self.tile_data.clone(),
            self.palette_picker.clone(),
            bg_mode.clone(),
            self.tilemap_editor.clone(),
        );

        // setup tilemap editor
        self.tilemap_editor.setup_all(
            self.palette_data.clone(),
            self.tile_data.clone(),
            self.tile_picker.imp().tile_size.clone(),
            self.palette_picker.clone(),
            self.tile_picker.clone(),
            self.obj().clone(),
        );
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
