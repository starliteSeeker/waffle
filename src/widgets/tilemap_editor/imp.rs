use std::cell::{Cell, RefCell};

use glib::subclass::InitializingObject;
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{CompositeTemplate, DrawingArea, DropDown, ScrolledWindow, StringList, ToggleButton};
use strum::IntoEnumIterator;

use crate::data::{
    list_items::{BGModeTwo, DrawMode, Zoom},
    tilemap::Tile,
};

#[derive(Properties, CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/tilemap_editor.ui")]
#[properties(wrapper_type = super::TilemapEditor)]
pub struct TilemapEditor {
    #[template_child]
    pub tilemap_scroll: TemplateChild<ScrolledWindow>,
    #[template_child]
    pub tilemap_drawing: TemplateChild<DrawingArea>,
    #[template_child]
    pub zoom_select: TemplateChild<DropDown>,
    #[template_child]
    pub zoom_level_list: TemplateChild<StringList>,
    #[template_child]
    pub mode_select: TemplateChild<DropDown>,
    #[template_child]
    pub mode_list: TemplateChild<StringList>,
    #[template_child]
    pub pen_draw_btn: TemplateChild<ToggleButton>,
    #[template_child]
    pub rect_fill_btn: TemplateChild<ToggleButton>,
    #[template_child]
    pub flip_x_btn: TemplateChild<ToggleButton>,
    #[template_child]
    pub flip_y_btn: TemplateChild<ToggleButton>,
    #[template_child]
    pub priority_btn: TemplateChild<ToggleButton>,

    #[property(get, set, builder(Zoom::default()))]
    tilemap_zoom: Cell<Zoom>,
    pub curr_tile: RefCell<Tile>,
    pub curr_drag: RefCell<DrawMode>,
}

// The central trait for subclassing a GObject
#[glib::object_subclass]
impl ObjectSubclass for TilemapEditor {
    const NAME: &'static str = "TilemapEditor";
    type Type = super::TilemapEditor;
    type ParentType = gtk::Box;

    fn class_init(klass: &mut Self::Class) {
        klass.bind_template();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

#[glib::derived_properties]
impl ObjectImpl for TilemapEditor {
    fn constructed(&self) {
        self.parent_constructed();

        // setup zoom dropdown
        for i in Zoom::iter() {
            self.zoom_level_list.append(&i.to_string());
        }
        self.zoom_select.set_selected(Zoom::default() as u32);

        // setup bg mode dropdown
        for i in BGModeTwo::iter() {
            self.mode_list.append(&i.to_string());
        }
    }
}
impl WidgetImpl for TilemapEditor {}
impl BoxImpl for TilemapEditor {}
