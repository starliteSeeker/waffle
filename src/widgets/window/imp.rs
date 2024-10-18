use std::cell::{Cell, RefCell};
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

use gio::{ActionEntry, SimpleActionGroup};
use glib::clone;
use glib::subclass::InitializingObject;
use glib::subclass::Signal;
use glib::ByteArray;
use glib::Properties;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{gio, glib, CompositeTemplate};

use crate::data::{
    color::Color,
    list_items::{BGModeTwo, Bpp, TileSize},
    palette::{Palette, RenameMePalette},
    tilemap::RenameMeTilemap,
    tiles::{RenameMeTileset, Tileset},
};
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

    // color picker properties
    #[property(get, set)]
    picker_color: RefCell<Option<ByteArray>>,

    // palette picker properties
    pub(super) palette_data_2: RefCell<RenameMePalette>,
    #[property(get, set)]
    palette_sel_idx: Cell<u8>,
    #[property(get, set, nullable)]
    palette_file: RefCell<Option<PathBuf>>,

    // tile picker properties
    pub(super) tileset_data_2: RefCell<RenameMeTileset>,
    #[property(get, set)]
    tileset_sel_idx: Cell<u32>,
    #[property(get, set, nullable)]
    tileset_file: RefCell<Option<PathBuf>>,

    // tilemap editor properties
    pub(super) tilemap_data: RefCell<RenameMeTilemap>,
    #[property(get, set, nullable)]
    tilemap_file: RefCell<Option<PathBuf>>,

    #[property(get, set, builder(Bpp::default()))]
    pub tile_bpp: Cell<Bpp>,
    #[property(get, set, builder(BGModeTwo::default()))]
    pub bg_mode: Cell<BGModeTwo>,
    #[property(get, set, builder(TileSize::default()))]
    pub tile_size: Cell<TileSize>,

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
        obj.set_picker_color(ByteArray::from(&Color::default().into_bytes()));

        self.color_picker.handle_action(&obj);
        self.color_picker.render_widget(&obj);

        self.palette_picker.handle_action(&obj);
        self.palette_picker.render_widget(&obj);

        self.tile_picker.handle_action(&obj);
        self.tile_picker.render_widget(&obj);

        self.tilemap_editor.handle_action(&obj);
        self.tilemap_editor.render_widget(&obj);

        // debug stuff
        /* TODO remove when finishsed */
        let action_debug = ActionEntry::builder("printstuff")
            .activate(clone!(@weak self as this => move |_, _, _| {
                println!("debug");
            }))
            .build();
        let actions = SimpleActionGroup::new();
        actions.add_action_entries([action_debug]);
        self.obj().insert_action_group("debug", Some(&actions));
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("set-color")
                    .param_types([u32::static_type(), u32::static_type(), u32::static_type()])
                    .build(),
                Signal::builder("palette-data-changed").build(),
                Signal::builder("tileset-data-changed").build(),
                Signal::builder("tilemap-data-changed").build(),
            ]
        })
    }
}

// Trait shared by all widgets
impl WidgetImpl for Window {}

// Trait shared by all windows
impl WindowImpl for Window {}

// Trait shared by all application windows
impl ApplicationWindowImpl for Window {}
