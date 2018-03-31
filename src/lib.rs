#![feature(proc_macro)]

extern crate serde_xml;
extern crate serde;
#[macro_use]
extern crate serde_derive;
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

pub fn load_from_str(map_str: &str) -> Result<Map, XmlError> {
    serde_xml::from_str(map_str)
}

pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Map, XmlError> {
    let mut file = File::open(path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    load_from_str(&content)
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Tile {
    pub id: u32,

    #[serde(deserialize_with="::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,
    pub image: Option<Image>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
pub struct Layer {
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub opacity: Option<f32>,

    pub data: Data,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageLayer {
    pub name: String,
    pub opacity: Option<f32>,
    pub offsetx: Option<f32>,
    pub offsety: Option<f32>,
    pub visible: Option<u8>,

    pub image: Image,

    #[serde(deserialize_with="::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
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

    #[serde(rename(deserialize="tileset"))]
    pub tilesets: Vec<Tileset>,

    #[serde(rename(deserialize="layer"))]
    pub layers: Vec<Layer>,

    #[serde(deserialize_with="::properties::deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,

    #[serde(rename(deserialize="objectgroup"))]
    pub objectgroups: Vec<Objectgroup>,

    #[serde(rename(deserialize="imagelayer"))]
    pub imagelayers: Vec<ImageLayer>,
}

#[derive(Debug)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

impl serde::Deserialize for Color {
    fn deserialize<D: serde::Deserializer>(deserializer: &mut D) -> Result<Color, D::Error> {
        use regex::Regex;

        let color_str: String = try!(serde::Deserialize::deserialize(deserializer));

        lazy_static! {
            static ref COLOR_REGEX: Regex =
                Regex::new(
                    r"(?x)#?
(?P<alpha>[:xdigit:]{2})?
(?P<red>[:xdigit:]{2})
(?P<green>[:xdigit:]{2})
(?P<blue>[:xdigit:]{2})"
                ).unwrap();
        }

        let caps = COLOR_REGEX.captures(&color_str)
            .ok_or(serde::Error::custom("color did not match regex"))?;

        let red = caps.name("red").ok_or(serde::Error::custom("could not deserialize red"))?;
        let green = caps.name("green").ok_or(serde::Error::custom("could not deserialize green"))?;
        let blue = caps.name("blue").ok_or(serde::Error::custom("could not deserialize blue"))?;
        let alpha = caps.name("alpha");

        let red = u8::from_str_radix(red, 16)
            .map_err(|e| serde::Error::custom(format!("could not parse red: {}", e)))?;
        let green = u8::from_str_radix(green, 16)
            .map_err(|e| serde::Error::custom(format!("could not parse green: {}", e)))?;
        let blue = u8::from_str_radix(blue, 16)
            .map_err(|e| serde::Error::custom(format!("could not parse blue: {}", e)))?;
        let alpha = alpha.map_or(Ok(255), |alph| {
                u8::from_str_radix(alph, 16)
                    .map_err(|e| serde::Error::custom(format!("could not parse alpha: {}", e)))
            })?;

        Ok(Color {
            r: red,
            g: green,
            b: blue,
            a: alpha,
        })
    }
}
