pub mod data;
pub mod utils;
pub mod widgets;

use gtk::prelude::*;
use gtk::{gio, glib};
use gtk::{Application, Builder};
use widgets::window::Window;

const APP_ID: &str = "com.example.waffle";

// default width of an 8x8 tile
const TILE_W: f64 = 24.0;

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("waffle.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_startup(on_startup);
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn on_startup(app: &Application) {
    let menubar =
        Builder::from_resource("/com/example/waffle/menus.ui").object::<gio::Menu>("menubar");
    app.set_menubar(menubar.as_ref());
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = Window::new(app);

    // Present window
    window.present();
}
