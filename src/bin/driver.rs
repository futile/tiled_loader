extern crate tiled_loader;

use tiled_loader::Map;

fn main() {
    let map = Map::load("assets/multi.tmx");
    println!("{:#?}", map);
}
