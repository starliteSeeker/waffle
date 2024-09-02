use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::OnceLock;

use glib::clone;
use glib::closure_local;
use glib::subclass::{InitializingObject, Signal};
use glib::Properties;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{CompositeTemplate, DrawingArea, DropDown, ScrolledWindow, StringList, ToggleButton};
use strum::IntoEnumIterator;

use crate::data::list_items::{BGMode, BGModeTwo, Zoom};
use crate::data::tilemap::{Tile, Tilemap};

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

    pub map_data: Rc<RefCell<Tilemap>>,
    pub zoom_level: Rc<RefCell<Zoom>>,
    pub curr_tile: Rc<RefCell<Tile>>,

    pub bg_mode: Rc<RefCell<BGMode>>,

    #[property(get, set, nullable)]
    pub file: RefCell<Option<PathBuf>>,

    pub curr_drag: RefCell<Option<((u32, u32), (u32, u32))>>,
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
        self.zoom_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                *this.zoom_level.borrow_mut() = Zoom::iter().nth(this.zoom_select.selected() as usize).expect("shouldn't happen");
                this.tilemap_drawing.set_content_width((crate::TILE_W * 32.0 * this.zoom_level.borrow().to_val()) as i32);
                this.tilemap_drawing.set_content_height((crate::TILE_W * 32.0 * this.zoom_level.borrow().to_val()) as i32);
                this.obj().emit_by_name::<()>("zoom-level-changed", &[]);
            }));

        // setup bg mode dropdown
        for i in BGModeTwo::iter() {
            self.mode_list.append(&i.to_string());
        }
        self.mode_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                *this.bg_mode.borrow_mut() = BGMode::Two(BGModeTwo::iter().nth(this.mode_select.selected() as usize).expect("shouldn't happen"));
                this.obj().emit_by_name::<()>("bg-mode-changed", &[]);
            }));

        self.obj().connect_closure(
            "tilemap-changed",
            false,
            closure_local!(|this: Self::Type| {
                this.imp().tilemap_drawing.queue_draw();
            }),
        );

        self.flip_x_btn
            .connect_active_notify(clone!(@weak self as this => move |btn| {
                this.curr_tile.borrow_mut().set_x_flip(btn.is_active());
            }));
        self.flip_y_btn
            .connect_active_notify(clone!(@weak self as this => move |btn| {
                this.curr_tile.borrow_mut().set_y_flip(btn.is_active());
            }));
        self.priority_btn
            .connect_active_notify(clone!(@weak self as this => move |btn| {
                this.curr_tile.borrow_mut().set_priority(btn.is_active());
            }));

        self.obj().connect_closure(
            "bg-mode-changed",
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
