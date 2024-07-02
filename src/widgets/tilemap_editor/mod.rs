mod imp;

use std::cell::RefCell;
use std::rc::Rc;

use glib::clone;
use glib::closure_local;
use gtk::glib;
use gtk::prelude::*;
use gtk::subclass::prelude::*;
use gtk::EventControllerMotion;
use gtk::GestureClick;

use crate::data::list_items::Zoom;
use crate::data::palette::Palette;
use crate::data::tiles::Tileset;

glib::wrapper! {
    pub struct TilemapEditor(ObjectSubclass<imp::TilemapEditor>)
        @extends gtk::Widget,
        @implements gtk::Accessible, gtk::Buildable, gtk::ConstraintTarget,
                    gtk::Orientable;
}

impl TilemapEditor {
    pub fn setup_all<P: WidgetExt, T: WidgetExt>(
        &self,
        palette_data: Rc<RefCell<Palette>>,
        tile_data: Rc<RefCell<Tileset>>,
        palette_obj: P,
        tile_obj: T,
    ) {
        self.setup_gesture();
        self.setup_draw(palette_data, tile_data.clone());
        self.setup_signal_connection(palette_obj, tile_obj, tile_data);
    }

    fn setup_gesture(&self) {
        let click_event = GestureClick::new();
        click_event.connect_released(clone!(@weak self as this => move |_, _, x, y| {
            let imp = this.imp();
            let xx = x + imp.tilemap_scroll.hadjustment().value();
            let yy = y + imp.tilemap_scroll.vadjustment().value();
            let tile_w = crate::TILE_W * imp.zoom_level.borrow().to_val();
            let new_idx = (yy / tile_w) as u32 * 32 + (xx / tile_w) as u32;
            println!("click on {new_idx}");
            // emit signal
            if imp.map_data.borrow_mut().set_tile(new_idx, &*imp.curr_tile.borrow()) {
                println!("change tile {}", imp.curr_tile.borrow().tile_idx);
                this.emit_by_name::<()>("tilemap-changed", &[]);
            }
        }));
        self.imp().tilemap_drawing.add_controller(click_event);

        // let hover_event = EventControllerMotion::new();
    }

    fn setup_draw(&self, palette_data: Rc<RefCell<Palette>>, tile_data: Rc<RefCell<Tileset>>) {
        let imp = self.imp();
        imp.tilemap_drawing.set_draw_func(
            clone!(@weak imp, @weak palette_data, @weak tile_data => move |_, cr, _, _| {
                let _ = cr.save();
                match *imp.zoom_level.borrow() {
                    Zoom::Half => cr.scale(0.5, 0.5),
                    Zoom::One => (),
                    Zoom::Two => cr.scale(2.0, 2.0),
                }
                imp.map_data.borrow().draw(cr, palette_data, tile_data);
                let _ = cr.restore();
            }),
        );
    }
    fn setup_signal_connection<P: WidgetExt, T: WidgetExt>(
        &self,
        palette_obj: P,
        tile_obj: T,
        tile_data: Rc<RefCell<Tileset>>,
    ) {
        palette_obj.connect_closure(
            "palette-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: P| {
                let Some(this) = this else {return};
                this.imp().tilemap_drawing.queue_draw();
            }),
        );
        palette_obj.connect_closure(
            "palette-idx-changed",
            false,
            closure_local!(@weak-allow-none self as this => move |_: P, new_idx: u8| {
                let Some(this) = this else {return};
                // TODO: account for tilemap setting
                this.imp().curr_tile.borrow_mut().palette = new_idx;
            }),
        );

        tile_obj.connect_closure(
            "tile-idx-changed",
            false,
            closure_local!(@weak-allow-none self as this, @weak-allow-none tile_data => move |_: T| {
                let Some(this) = this else {return};
                let Some(tile_data) = tile_data else {return};
                this.imp().curr_tile.borrow_mut().tile_idx = tile_data.borrow().get_idx();
                println!("new tile {}", this.imp().curr_tile.borrow().tile_idx);
            }),
        );
    }
}