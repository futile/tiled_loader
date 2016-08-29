#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![plugin(serde_macros)]

extern crate serde_xml;
extern crate serde;
extern crate regex;
extern crate base64;
extern crate byteorder;

use std::path::Path;
use std::fs::File;
use std::io::Read;

#[macro_use]
mod util;
mod data;
mod properties;

pub use data::Data;
pub use properties::Properties;

pub type XmlError = serde_xml::Error;

#[derive(Debug, Deserialize)]
pub struct Image {
    pub source: String,
    pub width: u32,
    pub height: u32,
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
    #[serde(rename(deserialize="tile"))]
    pub tiles: Vec<Tile>,
    pub image: Option<Image>,

    pub firstgid: u32,
    pub name: String,
    pub tilewidth: u32,
    pub tileheight: u32,
    pub tilecount: u32,
    pub columns: u32,
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


#[derive(Debug, Deserialize)]
pub struct Map {
    #[serde(rename(deserialize="tileset"))]
    pub tilesets: Vec<Tileset>,

    pub version: String,

    pub orientation: Orientation,

    pub renderorder: String,

    pub width: u32,
    pub height: u32,

    pub tilewidth: u32,
    pub tileheight: u32,

    pub nextobjectid: u32,

    #[serde(rename(deserialize="layer"))]
    pub layers: Vec<Layer>,

    #[serde(deserialize_with="::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,
}

impl Map {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Map, XmlError> {
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(serde_xml::from_str(&content)?)
    }
}
