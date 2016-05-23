use serde::de;
use regex::Regex;

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
            .map(|s| s.parse().unwrap()) // TODO: should not unwrap, but return an error
            .collect();

        Ok(Data {
            encoding: enc,
            compression: comp,
            tile_gids: gids,
        })
    }
}
