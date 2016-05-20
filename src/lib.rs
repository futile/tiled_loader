#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![plugin(serde_macros)]

extern crate serde_xml;
extern crate serde;

use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use serde::de;

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

pub type Properties = HashMap<String, Property>;

fn deserialize_properties<D: de::Deserializer>(deserializer: &mut D)
                                               -> Result<Option<Properties>, D::Error> {
    #[derive(Debug, Deserialize)]
    pub struct RawProperty {
        pub name: String,
        #[serde(rename(deserialize="type"))]
        #[serde(default)]
        pub type_: String,
        pub value: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct RawProperties {
        #[serde(rename(deserialize="property"))]
        properties: Vec<RawProperty>,
    }

    let raw_props: RawProperties = try!(de::Deserialize::deserialize(deserializer));
    let mut props = Properties::new();

    use std::num::ParseFloatError;
    use std::num::ParseIntError;
    use std::str::ParseBoolError;
    use std::error::Error;

    for raw_prop in raw_props.properties {
        if props.contains_key(&raw_prop.name) {
            return Err(de::Error::custom(format!("property '{}' was found twice", raw_prop.name)));
        }

        let val = match &raw_prop.type_[..] {
            "" | "string" => Property::String(raw_prop.value),
            "float" => {
                Property::Float(try!(raw_prop.value.parse().map_err(|e: ParseFloatError| {
                    de::Error::custom(e.description())
                })))
            }
            "bool" => {
                Property::Bool(try!(raw_prop.value.parse().map_err(|e: ParseBoolError| {
                    de::Error::custom(e.description())
                })))
            }
            "int" => {
                Property::Int(try!(raw_prop.value.parse().map_err(|e: ParseIntError| {
                    de::Error::custom(e.description())
                })))
            }
            s => return Err(de::Error::custom(format!("unexpected property type: '{}'", s))),
        };

        props.insert(raw_prop.name, val);
    }

    Ok(Some(props))
}

#[derive(Debug, Deserialize)]
pub struct Tile {
    id: u32,

    #[serde(deserialize_with="deserialize_properties")]
    #[serde(default)]
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
    Shifted,
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
    pub compression: Option<String>, /* #[serde(rename(deserialize="$value"))]
                                      * pub value: Option<String>, */
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

    #[serde(deserialize_with="deserialize_properties")]
    #[serde(default)]
    pub properties: Option<Properties>,
}

impl Map {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Map, XmlError> {
        let mut file = File::open(path)?;;
        let mut content = String::new();
        file.read_to_string(&mut content)?;;

        Ok(serde_xml::from_str(&content)?))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {}
}
