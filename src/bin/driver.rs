extern crate tiled_loader;

fn main() {
    let map = tiled_loader::load_from_path("assets/base64_gzip.tmx").unwrap();
    println!("{:#?}", map);
}
