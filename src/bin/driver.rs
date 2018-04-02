extern crate tiled_loader;
extern crate walkdir;

use walkdir::{DirEntry, WalkDir};

fn main() {
    for entry in WalkDir::new("assets") {
        let entry: DirEntry = entry.unwrap();
        if entry.file_type().is_file() && entry.file_name().to_str().unwrap().ends_with(".tmx") {
            match tiled_loader::load_from_path(entry.path()) {
                Ok(_) => println!("{}: success", entry.path().display()),
                Err(ref e) => println!("{}: {}", entry.path().display(), e),
            };
        }
    }
}
