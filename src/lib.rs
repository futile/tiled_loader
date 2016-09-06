#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![plugin(serde_macros)]

extern crate serde_xml;
extern crate serde;
extern crate regex;
extern crate base64;
extern crate byteorder;
extern crate flate2;
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

pub use data::{Data, DataEncoding, DataCompression};
pub use properties::Properties;
pub use objects::{Object, Objectgroup, Ellipse, Polyline, Polygon};

pub type XmlError = serde_xml::Error;

#[derive(Debug, Deserialize)]
pub struct Image {
    pub width: u32,
    pub height: u32,

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

    #[serde(deserialize_with="::properties::deserialize_properties")]
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

    #[serde(rename(deserialize="tile"))]
    pub tiles: Vec<Tile>,
    pub image: Option<Image>,
}

enum_str!(Orientation {
    Orthogonal("orthogonal"),
    Isometric("isometric"),
    Hexagonal("hexagonal"),
    Staggered("staggered"),
});

#[derive(Debug, Deserialize)]
pub struct Layer {
    pub name: String,
    pub width: u32,
    pub height: u32,

    pub data: Data,
}

pub type Color = String;

#[derive(Debug, Deserialize)]
pub struct Map {
    pub version: String,

    pub orientation: Orientation,

    pub renderorder: String,

    pub width: u32,
    pub height: u32,

    pub tilewidth: u32,
    pub tileheight: u32,

    pub nextobjectid: u32,

    pub backgroundcolor: Option<Color>,

    #[serde(rename(deserialize="tileset"))]
    pub tilesets: Vec<Tileset>,

    #[serde(rename(deserialize="layer"))]
    pub layers: Vec<Layer>,

    #[serde(deserialize_with="::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,

    #[serde(rename(deserialize="objectgroup"))]
    pub objectgroups: Vec<Objectgroup>,
}

pub fn load_from_str(map_str: &str) -> Result<Map, XmlError> {
    serde_xml::from_str(map_str)
}

pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Map, XmlError> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    load_from_str(&content)
}
