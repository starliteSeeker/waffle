use std::cell::RefCell;
use std::rc::Rc;
use std::sync::OnceLock;

use enum_iterator::all;
use glib::subclass::{InitializingObject, Signal};
use glib::{clone, closure_local};
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::{Button, CompositeTemplate, DrawingArea, DropDown, Label, StringList};

use crate::data::list_items::TileSize;

#[derive(CompositeTemplate, Default)]
#[template(resource = "/com/example/waffle/tile_picker.ui")]
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
        // klass.bind_template_callbacks();
    }

    fn instance_init(obj: &InitializingObject<Self>) {
        obj.init_template();
    }
}

impl ObjectImpl for TilePicker {
    fn constructed(&self) {
        self.parent_constructed();

        /*
        self.map_btn
            .connect_clicked(clone!(@weak self as this => move |_| {
                this.map_lbl.set_label(&this.map_data.get().to_string());
            }));
        */

        // populate StringList
        for i in all::<TileSize>() {
            self.tile_size_items.append(&format!("{}", i));
        }

        self.tile_size_select
            .connect_selected_notify(clone!(@weak self as this => move |_| {
                *this.tile_size.borrow_mut() = all::<TileSize>().nth(this.tile_size_select.selected() as usize).expect("shouldn't happen");
                this.obj().emit_by_name::<()>("tile-size-changed", &[]);
            }));

        // redraw self
        self.obj().connect_closure(
            "tile-size-changed",
            false,
            closure_local!(|this: Self::Type| {
                this.imp().tile_drawing.queue_draw();
            }),
        );
    }

    fn signals() -> &'static [Signal] {
        static SIGNALS: OnceLock<Vec<Signal>> = OnceLock::new();
        SIGNALS.get_or_init(|| {
            vec![
                Signal::builder("tile-idx-changed").build(),
                Signal::builder("tile-changed").build(),
                Signal::builder("tile-size-changed").build(),
            ]
        })
    }
}
impl WidgetImpl for TilePicker {}
impl BoxImpl for TilePicker {}
