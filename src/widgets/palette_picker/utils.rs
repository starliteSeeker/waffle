use std::fs::File;
use std::path::PathBuf;

use gtk::glib::{self, clone};

use crate::data::{file_format::PaletteFile, palette::Palette};
use crate::utils::*;
use crate::widgets::window::Window;

pub fn open_file(state: &Window, filepath: PathBuf, file_format: PaletteFile) {
    let file_result = match file_format {
        PaletteFile::BGR555 => Palette::from_file_bgr555(&filepath),
        PaletteFile::RGB24 => Palette::from_file_rgb24(&filepath),
    };
    match file_result {
        Ok(res) => {
            println!("load palette: {filepath:?}");
            state.set_palette_data(res);
            // only store file name (and allow reloading)
            // if the type is BGR555
            if file_format == PaletteFile::default() {
                state.set_palette_file(Some(filepath));
            } else {
                state.set_palette_file(None::<PathBuf>);
            }
            state.mark_palette_clean();
            state.clear_history();
        }
        Err(e) => eprintln!("Error: {e}"),
    }
}

pub fn save_file(state: &Window, filepath: PathBuf, file_format: PaletteFile) {
    match File::create(filepath.clone()) {
        Ok(f) => {
            match file_format {
                PaletteFile::BGR555 => {
                    let _ = state.palette_data().write_file_bgr555(&f);
                }
                PaletteFile::RGB24 => {
                    let _ = state.palette_data().write_file_rgb24(&f);
                }
            }
            println!("save palette: {filepath:?}");

            if file_format == PaletteFile::default() {
                state.set_palette_file(Some(filepath));
                state.mark_palette_clean();
            }
        }
        Err(e) => eprintln!("Error saving file: {e}"),
    }
}

pub fn unsaved_palette_dialog(state: &Window, after: impl Fn() + Clone + 'static) {
    let message = if let Some(file) = state.palette_file() {
        format!("Save palette changes to \"{}\"?", file.display())
    } else {
        "Save palette changes to new file?".to_string()
    };

    let after2 = after.clone();
    save_changes_dialog(
        state,
        message,
        clone!(@weak state => move || {
            let after1 = after.clone();
            if let Some(filepath) = state.palette_file() {
                // save to palette_file
                save_file(&state, filepath.clone(), PaletteFile::BGR555);
                after1();
            } else {
                // save to new file
                file_save_dialog(&state.clone(), move |_, filepath| {
                    save_file(&state, filepath, PaletteFile::BGR555);
                    after1();
                });
            }
        }),
        clone!(@weak state => move || {
            println!("discard unsaved palette");
            state.mark_palette_clean();
            after2();
        }),
    );
}
