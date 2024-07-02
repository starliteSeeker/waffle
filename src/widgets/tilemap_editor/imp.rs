use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use enum_iterator::all;
use glib::clone;
use glib::closure_local;
use glib::subclass::{InitializingObject, Signal};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{CompositeTemplate, DrawingArea, DropDown, ScrolledWindow, StringList};

use crate::data::list_items::{BGMode, Zoom};
use crate::data::tilemap::{Tile, Tilemap};

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/tilemap_editor.ui")]
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

    pub map_data: Rc<RefCell<Tilemap>>,
    pub zoom_level: Rc<RefCell<Zoom>>,
    pub curr_tile: Rc<RefCell<Tile>>,

    pub bg_mode: Rc<RefCell<BGMode>>,
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

impl ObjectImpl for TilemapEditor {
    fn constructed(&self) {
        self.parent_constructed();

        // setup zoom dropdown
        for i in all::<Zoom>() {
            self.zoom_level_list.append(&i.to_string());
        }
        self.zoom_select.set_selected(Zoom::default() as u32);
        self.zoom_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                *this.zoom_level.borrow_mut() = all::<Zoom>().nth(this.zoom_select.selected() as usize).expect("shouldn't happen");
                this.tilemap_drawing.set_content_width((crate::TILE_W * 32.0 * this.zoom_level.borrow().to_val()) as i32);
                this.tilemap_drawing.set_content_height((crate::TILE_W * 32.0 * this.zoom_level.borrow().to_val()) as i32);
                this.obj().emit_by_name::<()>("zoom-level-changed", &[]);
            }));

        // setup bg mode dropdown
        for i in all::<BGMode>() {
            self.mode_list.append(&i.to_string());
        }
        self.mode_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                *this.bg_mode.borrow_mut() = all::<BGMode>().nth(this.mode_select.selected() as usize).expect("shouldn't happen");
                this.obj().emit_by_name::<()>("bg-mode-changed", &[]);
            }));

        self.obj().connect_closure(
            "tilemap-changed",
            false,
            closure_local!(|this: Self::Type| {
                this.imp().tilemap_drawing.queue_draw();
            }),
        );
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("tilemap-changed").build(),
                Signal::builder("zoom-level-changed").build(),
                Signal::builder("bg-mode-changed").build(),
            ]
        })
    }
}
impl WidgetImpl for TilemapEditor {}
impl BoxImpl for TilemapEditor {}