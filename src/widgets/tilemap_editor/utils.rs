use std::fs::File;
use std::path::PathBuf;

use gtk::glib::{self, clone};

use crate::data::tilemap::Tilemap;
use crate::utils::*;
use crate::widgets::window::Window;

pub fn open_file(state: &Window, filepath: PathBuf) {
    match Tilemap::from_file(&filepath) {
        Ok(res) => {
            println!("load tilemap: {filepath:?}");
            state.set_tilemap_data(res);
            state.set_tilemap_file(Some(filepath));
            state.mark_tilemap_clean();
            state.clear_history();
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

pub fn save_file(state: &Window, filepath: PathBuf) {
    match File::create(filepath.clone()) {
        Ok(f) => {
            let _ = state.tilemap_data().write_to_file(&f);
            println!("save tilemap: {filepath:?}");
            state.set_tilemap_file(Some(filepath));
            state.mark_tilemap_clean();
        }
        Err(e) => eprintln!("Error saving file: {e}"),
    }
}

pub fn unsaved_tilemap_dialog(state: &Window, after: impl Fn() + Clone + 'static) {
    let message = if let Some(file) = state.tilemap_file() {
        format!("Save tilemap changes to \"{}\"?", file.display())
    } else {
        "Save tilemap changes to new file?".to_string()
    };

    let after2 = after.clone();
    save_changes_dialog(
        state,
        message,
        clone!(@weak state => move || {
            let after1 = after.clone();
            if let Some(filepath) = state.tilemap_file() {
                // save to palette_file
                save_file(&state, filepath.clone());
                after1();
            } else {
                // save to new file
                file_save_dialog(&state.clone(), move |_, filepath| {
                    save_file(&state, filepath);
                    after1();
                });
            }
        }),
        clone!(@weak state => move || {
            println!("discard unsaved tilemap");
            state.mark_tilemap_clean();
            after2();
        }),
    );
}
