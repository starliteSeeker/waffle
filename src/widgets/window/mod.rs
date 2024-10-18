mod imp;

use glib::ByteArray;
use glib::Object;
use gtk::Application;
use gtk::{gio, glib};
use gtk::{prelude::*, subclass::prelude::*};

use crate::data::{
    color::Color,
    list_items::{Bpp, TileSize},
    palette::RenameMePalette,
    tilemap::{RenameMeTilemap, Tile},
    tiles::RenameMeTileset,
};

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
    pub fn palette_data(&self) -> std::cell::Ref<RenameMePalette> {
        self.imp().palette_data_2.borrow()
    }
    pub fn set_palette_data(&self, pal: RenameMePalette) {
        *self.imp().palette_data_2.borrow_mut() = pal;
        self.emit_by_name::<()>("palette-data-changed", &[]);
    }
    pub fn modify_palette_data(&self, f: impl Fn(&mut RenameMePalette) -> bool) {
        if f(&mut self.imp().palette_data_2.borrow_mut()) {
            self.emit_by_name::<()>("palette-data-changed", &[]);
        }
    }
    pub fn connect_palette_data_notify(&self, f: impl Fn(&Self) + 'static) {
        self.connect_local("palette-data-changed", false, move |args| {
            f(args[0].get().unwrap());
            return None;
        });
    }

    pub fn tileset_data(&self) -> std::cell::Ref<RenameMeTileset> {
        self.imp().tileset_data_2.borrow()
    }
    pub fn modify_tileset_data(&self, f: impl Fn(&mut RenameMeTileset) -> bool) {
        if f(&mut self.imp().tileset_data_2.borrow_mut()) {
            self.emit_by_name::<()>("tileset-data-changed", &[]);
        }
    }
    pub fn connect_tileset_data_notify(&self, f: impl Fn(&Self) + 'static) {
        self.connect_local("tileset-data-changed", false, move |args| {
            f(args[0].get().unwrap());
            return None;
        });
    }

    pub fn tilemap_data(&self) -> std::cell::Ref<RenameMeTilemap> {
        self.imp().tilemap_data.borrow()
    }
    pub fn modify_tilemap_data(&self, f: impl Fn(&mut RenameMeTilemap) -> bool) {
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
        self.modify_tilemap_data(|RenameMeTilemap(map)| {
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

    // helpful functions
    pub fn palette_base(&self) -> u8 {
        match self.tile_bpp() {
            Bpp::Two => self.bg_mode().palette_offset(),
            Bpp::Four => 0,
        }
    }

    pub fn is_valid_tileset_idx(&self) -> bool {
        let tile_len = self.tileset_data().0.len();
        match self.tile_size() {
            TileSize::Eight => (self.tileset_sel_idx() as usize) < tile_len,
            TileSize::Sixteen => self.tileset_sel_idx() as usize + 16 + 1 < tile_len,
        }
    }
}
