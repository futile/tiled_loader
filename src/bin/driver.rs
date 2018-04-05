extern crate tiled_loader;
extern crate walkdir;

use walkdir::{DirEntry, WalkDir};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.len() == 2 {
        println!("{:#?}", tiled_loader::load_from_path(&args[1]));
    } else if args.len() == 1 {
        for entry in WalkDir::new("assets") {
            let entry: DirEntry = entry.unwrap();
            if entry.file_type().is_file() && entry.file_name().to_str().unwrap().ends_with(".tmx")
            {
                match tiled_loader::load_from_path(entry.path()) {
                    Ok(_) => println!("{}: success", entry.path().display()),
                    Err(ref e) => println!("{}: {}", entry.path().display(), e),
                };
            }
        }
    } else {
        println!("usage: ./{} [path-to-tmx]", &args[0]);
        std::process::exit(1);
    }
}
