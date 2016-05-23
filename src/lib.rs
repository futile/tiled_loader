#![feature(custom_derive, plugin)]
#![feature(question_mark)]
#![plugin(serde_macros)]

extern crate serde_xml;
extern crate serde;
extern crate regex;

use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use serde::de;

use regex::Regex;

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
                Property::Float(try!(raw_prop.value
                    .parse()
                    .map_err(|e: ParseFloatError| de::Error::custom(e.description()))))
            }
            "bool" => {
                Property::Bool(try!(raw_prop.value
                    .parse()
                    .map_err(|e: ParseBoolError| de::Error::custom(e.description()))))
            }
            "int" => {
                Property::Int(try!(raw_prop.value
                    .parse()
                    .map_err(|e: ParseIntError| de::Error::custom(e.description()))))
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

#[derive(Debug, PartialEq, Eq)]
pub enum DataEncoding {
    Base64,
    CSV,
    XML,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataCompression {
    None,
    Zlib,
}

#[derive(Debug)]
pub struct Data {
    pub encoding: DataEncoding,
    pub compression: DataCompression,

    pub tile_gids: Vec<u32>,
}

impl de::Deserialize for Data {
    fn deserialize<D: de::Deserializer>(deserializer: &mut D) -> Result<Data, D::Error> {
        use serde_xml::value::{self, Element, Content};

        let data_elem: Element = try!(de::Deserialize::deserialize(deserializer));
        println!("data: {:#?}", data_elem);

        let enc = match data_elem.attributes.get("encoding") {
            Some(v) => {
                if v.len() != 1 {
                    return Err(de::Error::custom(format!("expected exactly one encoding, got: '{:?}'", v)));
                }

                match &*v[0] {
                    "base64" => DataEncoding::Base64,
                    "csv" => DataEncoding::CSV,
                    s => return Err(de::Error::custom(format!("unexpected encoding: '{}'", s))),
                }
            },
            None => DataEncoding::XML,
        };

        let comp = match data_elem.attributes.get("compression") {
            Some(v) => {
                if v.len() != 1 {
                    return Err(de::Error::custom(format!("expected exactly one compression, got: '{:?}'", v)));
                }

                match &*v[0] {
                    "zlib" => DataCompression::Zlib,
                    s => return Err(de::Error::custom(format!("unexpected compression: '{}'", s))),
                }
            },
            None => DataCompression::None,
        };

        if comp != DataCompression::None {
            return Err(de::Error::custom("compression not yet supported"));
        }

        if enc != DataEncoding::CSV {
            return Err(de::Error::custom("other encoding than CSV not yet supported"));
        }

        let data_text = match data_elem.members {
            Content::Text(s) => s,
            _ => return Err(de::Error::custom("expected text inside data")),
        };

        let csv_regex = Regex::new(r"(\d+)").unwrap();
        let gids: Vec<u32> = csv_regex.captures_iter(&data_text)
            .map(|cap| cap.at(1).unwrap())
            .map(|s| s.parse().unwrap())
            .collect();

        Ok(Data {
            encoding: enc,
            compression: comp,
            tile_gids: gids,
        })
    }
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
        let mut file = File::open(path)?;
        let mut content = String::new();
        file.read_to_string(&mut content)?;

        Ok(serde_xml::from_str(&content)?)
    }
}

// #[cfg(test)]
// mod tests {
//     #[test]
//     fn it_works() {}
// }
