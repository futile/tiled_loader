extern crate tiled_loader;

use tiled_loader::Map;

fn main() {
    let map = Map::load("assets/base64_gzip.tmx").unwrap();
    println!("{:#?}", map);
}
