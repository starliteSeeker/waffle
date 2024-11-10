mod imp;

use glib::ByteArray;
use glib::Object;
use gtk::Application;
use gtk::{gio, glib};
use gtk::{prelude::*, subclass::prelude::*};

use crate::data::{
    color::Color,
    list_items::{Bpp, TileSize},
    palette::Palette,
    tilemap::{Tile, Tilemap},
    tiles::Tileset,
};
use crate::undo_stack::Operation;

glib::wrapper! {
    pub struct Window(ObjectSubclass<imp::Window>)
        @extends gtk::ApplicationWindow, gtk::Window, gtk::Widget,
        @implements gio::ActionGroup, gio::ActionMap, gtk::Accessible,
                    gtk::Buildable, gtk::ConstraintTarget, gtk::Native,
                    gtk::Root, gtk::ShortcutManager;
}

impl Window {
    pub fn new(app: &Application) -> Self {
        // Create new window
        Object::builder()
            .property("application", app)
            .property("show-menubar", true)
            .build()
    }

    // custom get/set/notify for non-properties
    pub fn palette_data(&self) -> std::cell::Ref<Palette> {
        self.imp().palette_data.borrow()
    }
    pub fn set_palette_data(&self, pal: Palette) {
        *self.imp().palette_data.borrow_mut() = pal;
        self.emit_by_name::<()>("palette-data-changed", &[]);
    }
    pub fn modify_palette_data(&self, f: impl Fn(&mut Palette) -> bool) {
        if f(&mut self.imp().palette_data.borrow_mut()) {
            self.emit_by_name::<()>("palette-data-changed", &[]);
        }
    }
    pub fn connect_palette_data_notify(&self, f: impl Fn(&Self) + 'static) {
        self.connect_local("palette-data-changed", false, move |args| {
            f(args[0].get().unwrap());
            return None;
        });
    }

    pub fn tileset_data(&self) -> std::cell::Ref<Tileset> {
        self.imp().tileset_data.borrow()
    }
    pub fn set_tileset_data(&self, tileset: Tileset) {
        *self.imp().tileset_data.borrow_mut() = tileset;
        self.emit_by_name::<()>("tileset-data-changed", &[]);
    }
    pub fn modify_tileset_data(&self, f: impl Fn(&mut Tileset) -> bool) {
        if f(&mut self.imp().tileset_data.borrow_mut()) {
            self.emit_by_name::<()>("tileset-data-changed", &[]);
        }
    }
    pub fn connect_tileset_data_notify(&self, f: impl Fn(&Self) + 'static) {
        self.connect_local("tileset-data-changed", false, move |args| {
            f(args[0].get().unwrap());
            return None;
        });
    }

    pub fn tilemap_data(&self) -> std::cell::Ref<Tilemap> {
        self.imp().tilemap_data.borrow()
    }
    pub fn set_tilemap_data(&self, tilemap: Tilemap) {
        *self.imp().tilemap_data.borrow_mut() = tilemap;
        self.emit_by_name::<()>("tilemap-data-changed", &[]);
    }
    pub fn modify_tilemap_data(&self, f: impl Fn(&mut Tilemap) -> bool) {
        if f(&mut self.imp().tilemap_data.borrow_mut()) {
            self.emit_by_name::<()>("tilemap-data-changed", &[]);
        }
    }
    pub fn connect_tilemap_data_notify(&self, f: impl Fn(&Self) + 'static) {
        self.connect_local("tilemap-data-changed", false, move |args| {
            f(args[0].get().unwrap());
            return None;
        });
    }
    pub fn put_tile(&self, idx: usize, tile: &Tile) {
        self.modify_tilemap_data(|Tilemap(map)| {
            let Some(old_tile) = map.get_mut(idx) else {
                return false;
            };
            if *old_tile != *tile {
                *old_tile = *tile;
                return true;
            } else {
                return false;
            }
        });
    }

    pub fn picker_color_inner(&self) -> Color {
        let curr_color = *self
            .picker_color()
            .expect("picker_color not initialized")
            .first_chunk()
            .expect("picker_color size mismatch");
        Color::from_bytes(curr_color)
    }
    pub fn modify_picker_color(&self, f: impl Fn(Color) -> Color) {
        let curr_color = self.picker_color_inner();
        let new_color = f(curr_color);
        if curr_color != new_color {
            self.set_picker_color(ByteArray::from(&new_color.into_bytes()));
        }
    }

    // undo stack
    pub fn push_op(&self, op: Operation) {
        self.imp().undo_stack.borrow_mut().push(op);
    }
    pub fn undo(&self) {
        self.imp().undo_stack.borrow_mut().undo();
    }
    pub fn redo(&self) {
        self.imp().undo_stack.borrow_mut().redo();
    }
    pub fn clear_history(&self) {
        println!("reset undo");
        self.imp().undo_stack.borrow_mut().clear();
    }
    pub fn palette_dirty(&self) -> bool {
        self.imp().undo_stack.borrow().palette_dirty()
    }
    pub fn mark_palette_clean(&self) {
        self.imp().undo_stack.borrow_mut().mark_palette_clean()
    }
    pub fn tilemap_dirty(&self) -> bool {
        self.imp().undo_stack.borrow().tilemap_dirty()
    }
    pub fn mark_tilemap_clean(&self) {
        self.imp().undo_stack.borrow_mut().mark_tilemap_clean()
    }

    // helpful functions
    // idx of palette 0 color 0
    pub fn palette_base(&self) -> u8 {
        match self.tile_bpp() {
            Bpp::Two => self.bg_mode().palette_offset(),
            Bpp::Four => 0,
        }
    }

    // check if a selected 8x8 or 16x16 tile is valid
    pub fn is_valid_tileset_idx(&self) -> bool {
        let tile_len = self.tileset_data().0.len();
        match self.tile_size() {
            TileSize::Eight => (self.tileset_sel_idx() as usize) < tile_len,
            TileSize::Sixteen => self.tileset_sel_idx() as usize + 16 + 1 < tile_len,
        }
    }

    // palette selected in palette picker
    pub fn curr_palette(&self) -> u8 {
        (self.palette_sel_idx().wrapping_sub(self.palette_base()) / self.tile_bpp().to_val()) % 8
    }
}
