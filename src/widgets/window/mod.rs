mod imp;

use glib::Object;
use glib::{BoxedAnyObject, ByteArray};
use gtk::Application;
use gtk::{gio, glib};

use crate::data::{
    color::Color,
    list_items::{Bpp, TileSize},
    palette::RenameMePalette,
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
            .property(
                "palette-data",
                Some(BoxedAnyObject::new(RenameMePalette::default())),
            )
            .property(
                "tileset-data",
                Some(BoxedAnyObject::new(RenameMeTileset::default())),
            )
            .build()
    }

    pub fn palette_base(&self) -> u8 {
        match self.tile_bpp() {
            Bpp::Two => self.bg_mode().palette_offset(),
            Bpp::Four => 0,
        }
    }

    pub fn is_valid_tileset_idx(&self) -> bool {
        let tile_len = self
            .tileset_data()
            .unwrap()
            .borrow::<RenameMeTileset>()
            .0
            .len();
        match self.tile_size() {
            TileSize::Eight => (self.tileset_sel_idx() as usize) < tile_len,
            TileSize::Sixteen => self.tileset_sel_idx() as usize + 16 + 1 < tile_len,
        }
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

    pub fn modify_palette_data(&self, f: impl Fn(&mut RenameMePalette) -> bool) {
        let palette_data = self.palette_data().unwrap();
        if f(&mut palette_data.borrow_mut::<RenameMePalette>()) {
            self.notify_palette_data();
        }
    }
}
