extern crate tiled_loader;

use tiled_loader::Map;

fn main() {
    let map = Map::load("assets/base64.tmx").unwrap();
    println!("{:#?}", map);
}
