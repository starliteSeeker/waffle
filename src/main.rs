pub mod widgets;

use gtk::prelude::*;
use gtk::{gio, glib, Application};
use widgets::window::Window;

const APP_ID: &str = "com.example.waffle";

fn main() -> glib::ExitCode {
    // Register and include resources
    gio::resources_register_include!("waffle.gresource").expect("Failed to register resources.");

    // Create a new application
    let app = Application::builder().application_id(APP_ID).build();

    // Connect to "activate" signal of `app`
    app.connect_activate(build_ui);

    // Run the application
    app.run()
}

fn build_ui(app: &Application) {
    // Create a window and set the title
    let window = Window::new(app);

    // Present window
    window.present();
}
