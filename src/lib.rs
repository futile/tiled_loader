#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![plugin(serde_macros)]

extern crate serde_xml;

use std::path::Path;
use std::fs::File;
use std::io::Read;

pub type XmlError = serde_xml::Error;

#[derive(Debug, Deserialize)]
pub struct Image {
    pub source: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct Property {
    pub name: String,
    #[serde(rename(deserialize="type"))]
    pub type_: Option<String>,
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Properties {
    #[serde(rename(deserialize="property"))]
    properties: Vec<Property>,
}

#[derive(Debug, Deserialize)]
pub struct Tile {
    id: u32,

    properties: Option<Properties>,
    image: Option<Image>,
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

#[derive(Debug, Deserialize)]
pub enum Orientation {
    Orthogonal,
    Isometric,
    Hexagonal,
    Shifted
}

#[derive(Debug, Deserialize)]
pub struct Layer {
    pub name: String,
    pub width: u32,
    pub height: u32,

    pub data: Data,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub encoding: Option<String>,
    pub compression: Option<String>,

    #[serde(rename(deserialize="$value"))]
    pub value: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Map {
    #[serde(rename(deserialize="tileset"))]
    pub tilesets: Vec<Tileset>,

    pub version: String,

    pub orientation: String,

    pub renderorder: String,

    pub width: u32,
    pub height: u32,

    pub tilewidth: u32,
    pub tileheight: u32,

    pub nextobjectid: u32,

    #[serde(rename(deserialize="layer"))]
    pub layers: Vec<Layer>,

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

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
