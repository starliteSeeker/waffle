use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::OnceLock;

use gio::{ActionEntry, SimpleActionGroup};
use glib::clone;
use glib::subclass::InitializingObject;
use glib::subclass::Signal;
use glib::Properties;
use glib::{variant::Variant, VariantTy};
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use crate::data::{color::Color, palette::Palette, tiles::Tileset};
use crate::widgets::{
    color_picker::ColorPicker, palette_picker::PalettePicker, tile_picker::TilePicker,
    tilemap_editor::TilemapEditor,
};

#[derive(CompositeTemplate, Properties, Default)]
#[template(resource = "/com/example/waffle/window.ui")]
#[properties(wrapper_type = super::Window)]
pub struct Window {
    #[template_child]
    pub color_picker: TemplateChild<ColorPicker>,
    #[template_child]
    pub palette_picker: TemplateChild<PalettePicker>,
    #[template_child]
    pub tilemap_editor: TemplateChild<TilemapEditor>,
    #[template_child]
    pub tile_picker: TemplateChild<TilePicker>,

    #[property(get, set, builder(VariantTy::ARRAY))]
    pub picker_color: RefCell<Option<Variant>>,

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
#[glib::derived_properties]
impl ObjectImpl for Window {
    fn constructed(&self) {
        self.parent_constructed();

        let obj = self.obj();

        // initialize variables
        obj.set_picker_color(Color::default().to_variant());

        let bg_mode = self.tilemap_editor.imp().bg_mode.clone();

        self.color_picker.handle_action(&obj);
        self.color_picker.render_widget(&obj);

        /*
        self.color_picker
            .setup_all(self.palette_picker.clone(), self.palette_data.clone());
        */

        self.palette_picker.setup_all(
            self.palette_data.clone(),
            self.obj().clone(),
            self.color_picker.clone(),
            bg_mode.clone(),
            self.tilemap_editor.clone(),
        );

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

        // debug stuff
        /* TODO remove when finishsed */
        let action_debug = ActionEntry::builder("printstuff")
            .activate(clone!(@weak self as this => move |_, _, _| {
                println!("debug.printstuff");
                let color = this.obj().picker_color().expect("picker_color setup error");
                println!("{:?}", Color::from_variant(&color));
            }))
            .build();
        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_debug]);
        self.obj().insert_action_group("debug", Some(&actions));
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
