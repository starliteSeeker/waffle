use std::path::PathBuf;

use gtk::prelude::*;
use gtk::{FileChooserAction, FileChooserDialog, FileFilter, ResponseType, Window};

pub fn file_open_dialog<W: IsA<Window>, F: Fn(PathBuf) + 'static>(parent: W, f: F) {
    let dialog = FileChooserDialog::new(
        Some("Open File"),
        Some(&parent),
        FileChooserAction::Open,
        &[("Cancel", ResponseType::Cancel), ("Open", ResponseType::Ok)],
    );

    // *.bin file filter and all file filter
    let bin_filter = FileFilter::new();
    bin_filter.set_name(Some("Binary Files (.bin)"));
    bin_filter.add_suffix("bin");
    let all_filter = FileFilter::new();
    all_filter.set_name(Some("All Files"));
    all_filter.add_pattern("*");
    dialog.add_filter(&bin_filter);
    dialog.add_filter(&all_filter);

    dialog.connect_response(move |d: &FileChooserDialog, response: ResponseType| {
        if response == ResponseType::Ok {
            // load file
            let file = d.file().expect("Couldn't get file");
            let filename = file.path().expect("Couldn't get file path");
            f(filename);
        }

        d.close();
    });

    dialog.show();
}

pub fn file_save_dialog<W: IsA<Window>, F: Fn(FileChooserDialog, PathBuf) + 'static>(
    parent: W,
    f: F,
) {
    let dialog = FileChooserDialog::new(
        Some("Save File"),
        Some(&parent),
        FileChooserAction::Save,
        &[
            ("Cancel", ResponseType::Cancel),
            ("Save", ResponseType::Accept),
        ],
    );

    dialog.connect_response(move |d: &FileChooserDialog, response: ResponseType| {
        if response == ResponseType::Accept {
            // load file
            let file = d.file().expect("Couldn't get file");
            let filename = file.path().expect("Couldn't get file path");
            f(d.clone(), filename);
        }

        d.close();
    });

    dialog.show();
}
