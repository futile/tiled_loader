use serde::de;
use regex::Regex;

use base64;
use byteorder::{ReadBytesExt, LittleEndian};

use serde_xml::value::{Element, Content};

use std::borrow::Cow;

#[derive(Debug, PartialEq, Eq)]
pub enum DataEncoding {
    Base64,
    CSV,
    XML,
}

impl DataEncoding {
    pub fn decode<E: de::Error>(&self,
                                data_content: &Content,
                                compression: &DataCompression)
                                -> Result<Vec<u32>, E> {
        match *self {
            DataEncoding::CSV => {
                let data_text = match data_content {
                    &Content::Text(ref s) => s,
                    _ => {
                        return Err(de::Error::custom("expected text inside data when decoding CSV"))
                    }
                };

                if compression != &DataCompression::None {
                    return Err(de::Error::custom("compression with csv-encoding not allowed"));
                }

                lazy_static! {
                    static ref CSV_REGEX: Regex = Regex::new(r"(\d+)").unwrap();
                }

                CSV_REGEX.captures_iter(&data_text)
                    .map(|cap| cap.at(1).unwrap())
                    .map(|s| {
                        s.parse()
                            .map_err(|e| de::Error::custom(format!("could not decode CSV: {}", e)))
                    })
                    .collect()
            }
            DataEncoding::Base64 => {
                let data_text = match data_content {
                    &Content::Text(ref s) => s.trim(),
                    _ => {
                        return Err(de::Error::custom("expected text inside data when decoding XML"))
                    }
                };

                let decoded_raw: Vec<u8> = try!(base64::decode(&data_text)
                    .map_err(|e| de::Error::custom(format!("could not decode base64: {}", e))));

                let decompressed = try!(compression.decompress(&decoded_raw.as_slice()));

                decompressed.chunks(4)
                    .map(|mut bytes| {
                        bytes.read_u32::<LittleEndian>().map_err(|e| {
                            de::Error::custom(format!("could not decode from little \
                                                       endian(base64): {}",
                                                      e))
                        })
                    })
                    .collect()
            }
            _ => return Err(de::Error::custom(format!("not yet supported encoding: '{:?}'", self))),
        }

    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum DataCompression {
    None,
    Zlib,
    Gzip,
}

impl DataCompression {
    fn decompress<'a, E: de::Error>(&self, compressed: &'a [u8]) -> Result<Cow<'a, [u8]>, E> {
        use std::io::Read;

        match *self {
            DataCompression::None => Ok(Cow::Borrowed(compressed)),
            DataCompression::Zlib => {
                use flate2::read::ZlibDecoder;

                let mut decoder = ZlibDecoder::new(compressed);
                let mut decoded = Vec::new();

                try!(decoder.read_to_end(&mut decoded).map_err(|e| {
                    de::Error::custom(format!("could not decode zlib-compressed data: {}", e))
                }));

                Ok(Cow::Owned(decoded))
            }
            DataCompression::Gzip => {
                use flate2::read::GzDecoder;

                let mut decoder = try!(GzDecoder::new(compressed).map_err(|e| {
                    de::Error::custom(format!("could not create gzip-decoder: {}", e))
                }));
                let mut decoded = Vec::new();

                try!(decoder.read_to_end(&mut decoded).map_err(|e| {
                    de::Error::custom(format!("could not decode gzip-compressed data: {}", e))
                }));

                Ok(Cow::Owned(decoded))
            }
        }
    }
}

#[derive(Debug)]
pub struct Data {
    pub encoding: DataEncoding,
    pub compression: DataCompression,

    pub tile_gids: Vec<u32>,
}

impl de::Deserialize for Data {
    fn deserialize<D: de::Deserializer>(deserializer: &mut D) -> Result<Data, D::Error> {
        let data_elem: Element = try!(de::Deserialize::deserialize(deserializer));

        let enc = match data_elem.attributes.get("encoding") {
            Some(v) => {
                if v.len() != 1 {
                    return Err(de::Error::custom(format!("expected exactly one encoding, got: \
                                                          '{:?}'",
                                                         v)));
                }

                match &*v[0] {
                    "base64" => DataEncoding::Base64,
                    "csv" => DataEncoding::CSV,
                    s => return Err(de::Error::custom(format!("unexpected encoding: '{}'", s))),
                }
            }
            None => DataEncoding::XML,
        };

        let comp = match data_elem.attributes.get("compression") {
            Some(v) => {
                if v.len() != 1 {
                    return Err(de::Error::custom(format!("expected exactly one compression, \
                                                          got: '{:?}'",
                                                         v)));
                }

                match &*v[0] {
                    "zlib" => DataCompression::Zlib,
                    "gzip" => DataCompression::Gzip,
                    s => return Err(de::Error::custom(format!("unexpected compression: '{}'", s))),
                }
            }
            None => DataCompression::None,
        };

        if enc == DataEncoding::XML {
            return Err(de::Error::custom("XML encoding not yet supported"));
        }

        let gids = try!(enc.decode(&data_elem.members, &comp));

        Ok(Data {
            encoding: enc,
            compression: comp,
            tile_gids: gids,
        })
    }
}
