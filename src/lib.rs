extern crate serde;
extern crate serde_xml_rs;
#[macro_use]
extern crate serde_derive;
extern crate base64;
extern crate byteorder;
extern crate flate2;
extern crate regex;
#[macro_use]
extern crate lazy_static;

use std::path::Path;
use std::fs::File;
use std::io::Read;

#[macro_use]
mod util;
mod data;
mod properties;
mod objects;
mod map;
mod color;

pub use data::{Data, DataCompression, DataEncoding};
pub use properties::Properties;
pub use objects::{Ellipse, Object, Objectgroup, Polygon, Polyline};

pub type XmlError = serde_xml_rs::Error;

pub fn load_from_str(map_str: &str) -> Result<Map, XmlError> {
    serde_xml_rs::from_str(map_str)
}

pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Map, XmlError> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    load_from_str(&content)
}

#[derive(Debug, Deserialize)]
pub struct Image {
    pub width: u32,
    pub height: u32,
    pub trans: Option<Color>,

    pub source: String,
}

#[derive(Debug)]
pub enum Property {
    String(String),
    Bool(bool),
    Float(f64),
    Int(i64),
}

#[derive(Debug, Deserialize)]
pub struct Tile {
    pub id: u32,

    #[serde(deserialize_with = "::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,
    pub image: Option<Image>,
}

#[derive(Debug, Deserialize)]
pub struct Tileset {
    pub firstgid: u32,
    pub name: String,
    pub tilewidth: u32,
    pub tileheight: u32,
    pub tilecount: u32,
    pub columns: u32,

    #[serde(rename(deserialize = "tile"), default)]
    pub tiles: Vec<Tile>,
    pub image: Option<Image>,
}

enum_str!(Orientation {
    Orthogonal("orthogonal"),
    Isometric("isometric"),
    Hexagonal("hexagonal"),
    Staggered("staggered"),
});

enum_str!(StaggerAxis {
    X("x"),
    Y("y"),
});

enum_str!(StaggerIndex {
    Even("even"),
    Odd("odd"),
});

enum_str!(TileRenderOrder {
    RightDown("right-down"),
    RightUp("right-up"),
    LeftDown("left-down"),
    LeftUp("left-up"),
});

#[derive(Debug, Deserialize)]
pub struct Layer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub opacity: Option<f32>,

    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct ImageLayer {
    pub name: String,
    pub opacity: Option<f32>,
    pub offsetx: Option<f32>,
    pub offsety: Option<f32>,
    pub visible: Option<u8>,

    pub image: Image,

    #[serde(deserialize_with = "::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,
}

#[derive(Debug)]
pub enum MapLayer {
    Layer(Layer),
    ObjectGroup(Objectgroup),
    ImageLayer(ImageLayer),
}

#[derive(Debug)]
pub struct Map {
    pub version: String,

    pub orientation: Orientation,
    pub renderorder: TileRenderOrder,
    pub hexsidelength: Option<i32>,
    pub staggeraxis: Option<StaggerAxis>,
    pub staggerindex: Option<StaggerIndex>,

    pub width: u32,
    pub height: u32,
    pub tilewidth: u32,
    pub tileheight: u32,

    pub nextobjectid: u32,
    pub backgroundcolor: Option<Color>,

    pub properties: Option<Properties>,

    pub tilesets: Vec<Tileset>,

    pub layers: Vec<MapLayer>,
}

#[derive(Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}
